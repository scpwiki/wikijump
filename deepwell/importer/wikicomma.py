import json
import os
import re
from datetime import datetime

from .constants import UNKNOWN_CREATION_DATE
from .generator import generate_seed
from .structures import *

from py7zr import SevenZipFile

REVISION_FILENAME_REGEX = re.compile(r"(\d+)\.txt")


class WikicommaImporter:
    __slots__ = (
        "generator",
        "directory",
        "replace_colon",
    )

    def __init__(self, generator, directory, replace_colon = True):
        self.generator = generator
        self.directory = directory
        self.replace_colon = replace_colon

    def process_all(self):
        self.generator.section_sql("Wikicomma")
        self.generator.section_sh("Files")

        for site_slug in os.listdir(self.directory):
            self.process_site(site_slug)

    def process_site(self, site_slug):
        self.generator.section_sql(f"Site: {site_slug}")

        # Add site
        unknown_description = f"[NEEDS UPDATE] {site_slug}"
        self.generator.add_site(
            Site(
                wikidot_id=None,
                created_at=UNKNOWN_CREATION_DATE,
                name=unknown_description,
                slug=site_slug,
                subtitle=unknown_description,
                description=unknown_description,
            )
        )

        # Process site internals
        site_directory = os.path.join(self.directory, site_slug)
        self.process_site_pages(site_slug, site_directory)
        self.process_site_forum(site_slug, site_directory)

    def process_site_pages(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Pages: {site_slug}")
        page_mapping = self.read_json(site_directory, "meta", "page_id_map.json")
        file_mapping = self.read_json(site_directory, "meta", "file_map.json")

        def get_first_last_revisions(revisions: List[dict]):
            # Since the revision list isn't always in order...
            start_revision = revisions[0]
            last_revision = revisions[0]

            for revision in revisions:
                if revision["revision"] < start_revision["revision"]:
                    start_revision = revision

                if revision["revision"] > last_revision["revision"]:
                    last_revision = revision

            return start_revision, last_revision

        for page_id, page_slug in page_mapping.items():
            self.generator.section_sql(f"Page: {page_slug}")
            page_id = int(page_id)
            metadata = self.read_page_metadata(site_directory, page_slug)
            start_revision, last_revision = get_first_last_revisions(metadata["revisions"])
            created_at = datetime.fromtimestamp(start_revision["stamp"])
            updated_at = datetime.fromtimestamp(last_revision["stamp"])
            site_id = -1 # TODO unknown

            self.generator.add_page(
                Page(
                    wikidot_id=page_id,
                    created_at=created_at,
                    updated_at=updated_at,
                    site_id=site_id,
                    title=metadata["title"],
                    slug=page_slug,
                    discussion_thread_id=None,  # TODO unknown
                )
            )
            self.generator.add_page_lock(page_id, metadata["is_locked"])
            self.process_page_revisions(site_directory, site_id, metadata)
            self.process_page_files(
                site_directory, page_id, file_mapping, metadata["files"],
            )
            self.process_page_votes(metadata)

    def process_page_revisions(self, site_directory: str, site_id: int, metadata: dict):
        page_slug = metadata["name"]
        page_id = metadata["page_id"]
        title = metadata["title"]  # We don't know what these are historically
        tags = metadata["tags"]

        wikitext_mapping = {}
        with self.open_page_revisions(site_directory, page_slug) as archive:
            for filename, data in archive.readall().items():
                match = REVISION_FILENAME_REGEX.fullmatch(filename)
                revision_number = int(match[1])
                wikitext = data.read().decode("utf-8")

        for revision in metadata["revisions"]:
            revision_number = revision["revision"]
            user_spec = revision["author"]

            # Is user slug, not a user ID
            if isinstance(user_spec, str):
                # TODO get ID
                continue

            wikitext = wikitext_mapping.get(revision_number)
            if wikitext is None:
                continue

            self.generator.add_page_revision(
                PageRevision(
                    wikidot_id=revision["global_revision"],
                    revision_number=revision_number,
                    created_at=datetime.fromtimestamp(revision["stamp"]),
                    flags=revision["flags"],
                    page_id=page_id,
                    site_id=site_id,
                    user_id=user_spec,
                    wikitext=wikitext,
                    slug=page_slug,
                    title=title,
                    html="",  # TODO not stored
                    tags=tags,
                    comments=revision["commentary"],
                )
            )

    def process_page_files(
        self, site_directory: str, page_id: int, file_mapping: dict, metadata_list: list,
    ):
        for metadata in metadata_list:
            user_spec = metadata["author"]
            # Is user slug, not a user ID
            if isinstance(user_spec, str):
                # TODO get ID
                continue

            file_location = file_mapping[str(metadata["file_id"])]
            file_path = os.path.join(site_directory, file_location["path"])

            with open(file_path, "rb") as file:
                file_data = file.read()

            self.generator.add_file(
                File(
                    wikidot_id=metadata["file_id"],
                    page_id=page_id,
                    name=metadata["name"],
                    mime=metadata["mime"],
                    size=metadata["size_bytes"],
                    user_id=user_spec,
                    created_at=datetime.fromtimestamp(metadata["stamp"]),
                )
            )

    def process_page_votes(self, metadata: dict):
        for (user_spec, value) in metadata["votings"]:
            # Is user slug, not a user ID
            if isinstance(user_spec, str):
                # TODO get ID
                continue

            # Get vote value
            if isinstance(value, bool):
                value = +1 if value else -1

            self.generator.add_page_vote(
                PageVote(
                    page_id=metadata["page_id"],
                    user_id=user_spec,
                    value=value,
                )
            )

    def process_site_forum(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Forum: {site_slug} [TODO]")
        # TODO

    def read_page_metadata(self, site_directory: str, page_slug: str):
        page_metadata_filename = f"{page_slug}.json"

        if self.replace_colon:
            page_metadata_filename = page_metadata_filename.replace(":", "_")

        page_metadata = self.read_json(
            site_directory,
            "meta",
            "pages",
            page_metadata_filename,
        )

        assert page_metadata["name"] == page_slug
        return page_metadata

    def open_page_revisions(self, site_directory: str, page_slug: str):
        page_revisions_filename = f"{page_slug}.7z"

        if self.replace_colon:
            page_revisions_filename = page_revisions_filename.replace(":", "_")

        page_revisions_path = os.path.join(site_directory, "pages", page_revisions_filename)
        return SevenZipFile(page_revisions_path, "r")

    @staticmethod
    def read_json(*path_parts):
        path = os.path.join(*path_parts)

        with open(path) as file:
            return json.load(file)


def run_wikicomma_import(
    *,
    wikicomma_directory: str,
    sql_path: str,
    sh_path: str,
    s3_bucket: str,
    postgres_url: str,
    last_page_category_id: int = 0,
):
    wikicomma_directory = os.path.normpath(wikicomma_directory)

    def runner(generator):
        importer = WikicommaImporter(generator, wikicomma_directory)
        importer.process_all()

    generate_seed(
        runner,
        sql_path=sql_path,
        sh_path=sh_path,
        s3_bucket=s3_bucket,
        postgres_url=postgres_url,
        last_page_category_id=last_page_category_id,
    )
