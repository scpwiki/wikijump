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
        site = Site(
            wikidot_id=None,
            created_at=UNKNOWN_CREATION_DATE,
            name=unknown_description,
            slug=site_slug,
            subtitle=unknown_description,
            description=unknown_description,
        )
        self.generator.add_site(site)

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

            page = Page(
                wikidot_id=page_id,
                created_at=created_at,
                updated_at=updated_at,
                site_id=-1, # TODO unknown
                title=metadata["title"],
                slug=page_slug,
                discussion_thread_id=None,  # TODO unknown
            )
            self.generator.add_page(page)

    def process_site_forum(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Forum: {site_slug}")
        # TODO

    def process_site_files(self, site_slug: str, site_directory: str):
        self.generator.section_sql(f"Files: {site_slug}")
        # TODO

    def _add_page_votes(self, site, page, metadata):
        for (user_spec, value) in metadata["votings"]:
            user_id = self.get_user_id(user_spec)

            if isinstance(value, bool):
                value = +1 if value else -1

            self.append_sql(
                "INSERT INTO page_vote (created_at, page_id, user_id, value)",
                (UNKNOWN_CREATION_DATE, page.wikidot_id, user_id, value),
            )
        # lock: metadata["is_locked"]:

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


# XXX

    def add_site_pages(self, site):
        print(f"++ Writing pages")
        self.append_sql_section("Pages")

        # Load page mapping
        mapping = self.read_json(site.directory, "meta", "page_id_map.json")

        for page_id, page_slug in mapping.items():
            # Store page data locally
            page_id = int(page_id)
            self.pages[page_id] = page_slug

            # Add page to database
            page_category_id = self.add_page_category(page_slug)
            discussion_thread_id = None  # TODO get discussion thread ID

            sql.add_page(
                site, page_id, page_slug, created_at, updated_at, discussion_thread,
            )
            self.append_sql(
                "INSERT INTO page (page_id, created_at, updated_at, site_id, page_category_id, slug, discussion_thread_id) VALUES (%s, %s, %s, %s, %s, %s, %s)",
                (
                    page_id,
                    created_at,
                    updated_at,
                    site.wikijump_id,
                    page_category_id,
                    page_slug,
                    discussion_thread_id,
                ),
            )
            page = Page(slug=page_slug, wikidot_id=page_id)
            page_metadata = self.read_page_metadata(site, page_slug)

            # Add page components to database
            self.add_page_revisions(site, page, page_metadata)
            self.add_page_votes(site, page, page_metadata)
            self.add_page_lock(site, page, page_metadata)

    def add_page_revisions(self, site, page, metadata):
        for revision in metadata["revisions"]:
            user_id = self.get_user_id(revision["author"])
            title = metadata["title"]
            tags = metadata["tags"]

            self.append_sql(
                "INSERT INTO page_revision (revision_id, revision_type, created_at, revision_number, slug, page_id, site_id, user_id, changes, wikitext_hash, compiled_hash, compiled_at, compiled_generator, comments, title, tags) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)",
                (
                    revision["global_revision"],
                    revision_type,
                    datetime.fromtimestamp(revision["stamp"]),
                    revision["revision"],
                    page_slug,
                    page_id,
                    site.wikijump_id,
                    user_id,
                    changes,
                    wikitext_hash,
                    compiled_hash,
                    datetime.utcnow(),
                    "WikiComma import tool",
                    revision["commentary"],
                    title,  # NOTE: We can't tell what they were historically, so we just assign the same value
                    tags,
                ),
            )

    def add_site_forum(self, site):
        print(f"++ Writing forum posts")
        self.append_sql_section("Forum")

        # TODO
        print("++ TODO")

    def add_site_files(self, site):
        print(f"++ Writing files")
        self.append_sql_section("Files")

        # Load file mapping
        mapping = self.read_json(site.directory, "meta", "file_map.json")

        # TODO
        for file_id, file_data in mapping.items():
            # TODO
            # int(file_id)
            # file_data['url']
            # file_data['path']
            pass
