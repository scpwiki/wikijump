#!/usr/bin/env python3

import argparse
import hashlib
import os

import psycopg2


class WikicommaImporter:
    __slots__ = (
        "output_file",
        "database_url",
        "wikicomma_directory",
        "text_hashes",
        "_file",
        "_conn",
        "_cur",
    )

    def __init__(self, args):
        self.output_file = args.output_file
        self.database_url = args.database_url
        self.wikicomma_directory = os.path.normpath(args.wikicomma_directory)

        self.text_hashes = set()

        self._clean()

    def run(self):
        print("Preparing...")

        with open(self.output_file, "w") as self._file:
            with psycopg2.connect(self.database_url) as self._conn:
                with conn.cursor() as self._cur:
                    self.pull_all()

        self._clean()
        print(f"Finished. Wrote SQL query to {self.output_file}")

    def pull_all(self):
        for site in os.listdir(self.wikicomma_directory):
            self.pull_site(site)

    def pull_site(self, site):
        print(f"+ Pulling site {site}")
        site_directory = os.path.join(self.wikicomma_directory, site)

        self.pull_site_pages(site_directory)
        self.pull_site_forum(site_directory)
        self.pull_site_files(site_directory)

    def pull_site_pages(self, site_directory):
        print(f"++ Writing pages")
        self.append_sql_section("Pages")

        # TODO

    def pull_site_forum(self, site_directory):
        print(f"++ Writing forum posts")
        self.append_sql_section("Forum")

        # TODO
        print("++ TODO")

    def pull_site_files(self, site_directory):
        print(f"++ Writing files")
        self.append_sql_section("Files")

        # TODO

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

    def add_text(self, text):
        text_bytes = text.encode("utf-8")
        text_hash = hashlib.sha512(text_bytes).digest()

        if text_hash not in self.text_hashes:
            self.append_sql("INSERT INTO text (hash, contents) VALUES (%s, %s)", (text_hash, text))
            self.text_hashes.add(text_hash)

        return text_hash

if __name__ == "__main__":
    # Parse arguments
    argparser = argparse.ArgumentParser(description="WikiComma import utility")
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
