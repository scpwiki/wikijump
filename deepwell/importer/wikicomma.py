import json
import os
from datetime import datetime

from .constants import UNKNOWN_CREATION_DATE
from .generator import generate_seed
from .structures import *

import py7zr
from bidict import bidict

REPLACE_COLON = True
ANONYMOUS_USER_ID = 3


class WikicommaImporter:
    __slots__ = (
        "generator",
        "directory",
    )

    def __init__(self, generator, directory):
        self.generator = generator
        self.directory = directory

    def process_all(self):
        self.generator.section_sql("Wikicomma")
        self.generator.section_sh("Files")

        for site_slug in os.listdir(self.directory):
            self.process_site(site_slug)

    def process_site(self, site_slug):
        self.generator.section_sql(f"Site: {site_slug}")

        # Add site
        unknown_description = f"[NEEDS UPDATE] {site_slug}"
        self.generator.add_site(Site(
            wikidot_id=None,
            created_at=UNKNOWN_CREATION_DATE,
            name=unknown_description,
            slug=site_slug,
            subtitle=unknown_description,
            description=unknown_description,
        ))

        # Process site internals
        site_directory = os.path.join(self.directory, site_slug)
        self.process_site_pages(site_slug, site_directory)
        self.process_site_forum(site_slug, site_directory)
        self.process_site_files(site_slug, site_directory)

    def process_site_pages(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Pages: {site_slug}")
        mapping = self.read_json(site_directory, "meta", "page_id_map.json")

        for page_id, page_slug in mapping.items():
            metadata = self.read_page_metadata(site_directory, page_slug)
            created_at = datetime.fromtimestamp(metadata.revisions[-1]["stamp"])
            updated_at = datetime.fromtimestamp(metadata.revisions[0]["stamp"])

            self.generator.add_page(Page(
                wikidot_id=page_id,
                created_at=created_at,
                updated_at=updated_at,
                site_id=-1, # TODO unknown
                title=metadata["title"],
                slug=page_slug,
                discussion_thread_id=None,  # TODO unknown
            ))
            self.generator.add_page_lock(page_id, metadata["is_locked"])
            self.process_page_votes(metadata)

    def process_page_revisions(self, site_id: int, metadata: dict):
        page_id = metadata["page_id"]
        title = metadata["title"] # We don't know what these are historically
        tags = metadata["tags"]

        for revision in metadata["revisions"]:
            user_spec = revision["author"]

            # Is user slug, not a user ID
            if isinstance(user_spec, str):
                # TODO get ID
                continue

            self.generator.add_page_revision(PageRevision(
                wikidot_id=revision["global_revision"],
                revision_number=revision["revision"],
                created_at=datetime.fromtimestamp(revision["stamp"]),
                flags=revision["flags"],
                page_id=page_id,
                site_id=site_id,
                user_id=user_spec,
                wikitext=wikitext,
                slug=page_slug,
                html="", # TODO not stored
                tags=tags,
                comments=revision["commentary"],
            ))

    def process_page_votes(self, metadata: dict):
        for (user_spec, value) in metadata["votings"]:
            # Is user slug, not a user ID
            if isinstance(user_spec, str):
                # TODO get ID
                continue

            # Get vote value
            if isinstance(value, bool):
                value = +1 if value else -1

            self.generator.add_page_vote(PageVote(
                page_id=metadata["page_id"],
                user_id=user_spec,
                value=value,
            ))

    def process_site_forum(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Forum: {site_slug} [TODO]")
        # TODO

    def process_site_files(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Files: {site_slug}")
        mapping = self.read_json(site_directory, "meta", "file_map.json")

        for file_id, file_info in mapping.items():
            file_id = int(file_id)
            file_path = os.path.join(site_directory, file_info["path"])

            with open(file_path, "rb") as file:
                file_data = file.read()

            self.generator.add_file(...)

            # TODO
            # int(file_id)
            # file_data['url']
            # file_data['path']
            pass

    def read_page_metadata(self, site_directory: str, page_slug: str):
        page_metadata_filename = f"{page_slug}.json"

        if REPLACE_COLON:
            page_metadata_filename = page_metadata_filename.replace(":", "_")

        page_metadata = self.read_json(
            site_directory,
            "meta",
            "pages",
            page_metadata_filename,
        )

        assert page_metadata["name"] == page_slug
        return page_metadata

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
