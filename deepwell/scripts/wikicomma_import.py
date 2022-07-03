#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
from collections import namedtuple
from datetime import datetime

import psycopg2
import py7zr
from bidict import bidict

REPLACE_COLON = True
ANONYMOUS_USER_ID = 3
UNKNOWN_CREATION_DATE = datetime.fromtimestamp(0)

Site = namedtuple("Site", ("slug", "wikijump_id", "directory"))
Page = namedtuple("Page", ("slug", "wikidot_id"))


class WikicommaImporter:
    __slots__ = (
        "output_file",
        "database_url",
        "wikicomma_directory",
        "text_hashes",
        "pages",
        "page_categories",
        "last_site_id",
        "last_category_id",
        "_file",
        "_conn",
        "_cur",
    )

    def __init__(self, args):
        self.output_file = args.output_file
        self.database_url = args.database_url
        self.wikicomma_directory = os.path.normpath(args.wikicomma_directory)

        self.text_hashes = set()
        self.pages = bidict()
        self.page_categories = {}
        self.last_site_id = args.start_site_id
        self.last_category_id = args.start_category_id

        self._clean()

    def run(self):
        print("Preparing...")

        with open(self.output_file, "w") as self._file:
            with psycopg2.connect(self.database_url) as self._conn:
                with conn.cursor() as self._cur:
                    self.add_all()

        self._clean()
        print(f"Finished. Wrote SQL query to {self.output_file}")

    def add_all(self):
        for site_slug in os.listdir(self.wikicomma_directory):
            self.add_site(site_slug)

    def add_site(self, site_slug):
        print(f"+ Pulling site {site_slug}")

        # Create site
        self.append_sql_section(f"Site {site_slug}")
        self.append_sql(
            # NOTE: Site name, description, and date created will need to be adjusted
            "INSERT INTO site (slug, name, description, date_created)",
            (
                site_slug,
                f"[NEEDS UPDATE] {site_slug}",
                f"[NEEDS UPDATE] {site_slug}",
                UNKNOWN_CREATION_DATE,
            ),
        )

        # Reset site-specific values
        self.pages = bidict()
        self.page_categories = {}

        # Create site data objects
        site_id = self.next_site_id()
        site_directory = os.path.join(self.wikicomma_directory, site_slug)
        site = Site(slug=site_slug, wj_id=site_id, directory=site_directory)

        # Pull contents within this site
        self.add_site_pages(site)
        self.add_site_forum(site)
        self.add_site_files(site)

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
            user_id = None  # TODO get based on username, revision["author"]
            # if isinstance int, then that's the user ID
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

    def add_page_votes(self, site, page, metadata):
        for (user_spec, value) in metadata["votings"]:
            user_id = None  # TODO get based on username, user_spec
            # if isinstance int, then that's the user ID

            if isinstance(value, bool):
                value = +1 if value else -1

            self.append_sql(
                "INSERT INTO page_vote (created_at, page_id, user_id, value)",
                (UNKNOWN_CREATION_DATE, page.wikidot_id, user_id, value),
            )

    def add_page_lock(self, site, page, metadata):
        if metadata["is_locked"]:
            self.append_sql(
                "INSERT INTO page_lock (created_at, lock_type, page_id, user_id, reason) VALUES (%s, %s, %s, %s, %s)",
                (
                    UNKNOWN_CREATION_DATE,
                    "wikidot",
                    page.wikidot_id,
                    ANONYMOUS_USER_ID,
                    "Lock imported from Wikidot",
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

    def _clean(self):
        self._file = None
        self._file = None
        self._cur = None

    def format_sql(self, query, parameters=()):
        return self._cur.mogrify(query, parameters)

    def append_sql(self, query, parameters=()):
        sql_line = self.format_sql(query, parameters)
        self._file.writelines([sql_line, ";\n"])

    def append_sql_section(self, name):
        self._file.writelines(["\n\n--\n-- ", name, "\n--\n\n"])

    @staticmethod
    def get_page_category(page_slug):
        parts = page_slug.split(":")
        if len(parts) == 1:
            return "_default"

        return parts[0]

    def add_page_category(self, page_slug):
        category_slug = self.get_page_category(page_slug)

        if category_slug not in self.page_categories:
            self.page_categories[category_slug] = self.next_category_id()

        return self.page_categories[category_slug]

    def add_text(self, text):
        text_bytes = text.encode("utf-8")
        text_hash = hashlib.sha512(text_bytes).digest()

        if text_hash not in self.text_hashes:
            self.append_sql(
                "INSERT INTO text (hash, contents) VALUES (%s, %s)", (text_hash, text)
            )
            self.text_hashes.add(text_hash)

        return text_hash

    def add_user(self):
        # TODO
        pass

    def get_user(self, spec):
        # TODO
        pass

    def read_page_metadata(self, site, page_slug):
        page_metadata_filename = f"{page_slug}.json"

        if REPLACE_COLON:
            page_metadata_filename = page_metadata_filename.replace(":", "_")

        page_metadata = self.read_json(
            site.directory, "meta", "pages", page_metadata_filename,
        )

        assert page_metadata["name"] == page_slug
        return page_metadata

    @staticmethod
    def read_json(*path_parts):
        path = os.path.join(*path_parts)

        with open(path) as file:
            return json.load(file)

    def next_site_id(self):
        next_id = self.last_site_id
        self.last_site_id += 1
        return next_id

    def next_category_id(self):
        next_id = self.last_category_id
        self.last_category_id += 1
        return next_id


if __name__ == "__main__":
    # Parse arguments
    argparser = argparse.ArgumentParser(description="WikiComma import utility")
    argparser.add_argument(
        "--start-site-id",
        dest="start_site_id",
        default=3,
        help="What ID value to start enumerating new sites from",
    )
    argparser.add_argument(
        "--start-category-id",
        dest="start_category_id",
        default=3,
        help="What ID value to start enumerating new page categories from",
    )
    argparser.add_argument(
        "-o",
        "--output",
        dest="output_file",
        default="wikicomma_ingest.sql",
        help="The path to write the output SQL file to.",
    )
    argparser.add_argument(
        "database_url",
        help=(
            "A PostgreSQL connection string which has a DEEPWELL database. "
            "The database is not written to."
        ),
    )
    argparser.add_argument(
        "wikicomma_directory",
        help=(
            "The directory containing WikiComma data. "
            "Each top-level directory contains one wiki to be imported."
        ),
    )
    args = argparser.parse_args()

    # Run importer
    WikicommaImporter(args).run()
