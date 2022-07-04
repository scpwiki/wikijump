from collections import namedtuple

from bidict import bidict

from .constants import UNKNOWN_CREATION_DATE
from .counter import IncrementingCounter

import psycopg2

User = namedtuple("User", ("slug", "wikidot_id"))
Site = namedtuple("Site", ("slug", "wikijump_id"))


class SqlGenerator:
    """
    Generates SQL to be appended to the output file.

    Also tracks the state of all imported Wikidot data,
    as encountered. This is necessary to avoid inserting
    duplicate data.
    """

    def __init__(self, buffer, cursor, *, start_site_id):
        self.buffer = buffer
        self.cursor = cursor

        self.users = bidict()
        self.sites = {}
        self.site_counter = IncrementingCounter(start_site_id)
        self.text_hashes = set()

    def format(self, query, parameters=()):
        return self.cursor.mogrify(query, parameters)

    def append(self, query, parameters=()):
        sql_line = self.format(query, parameters)
        self.buffer.writelines([sql_line, ";\n"])

    def section(self, name):
        self.buffer.writelines(["\n\n--\n-- ", name, "\n--\n\n"])

    def add_user(self, user):
        # TODO implement method
        # TODO change over when user table changes
        raise NotImplementedError

    def add_site(self, site_slug):
        site = self.context.get_site(site_slug)

        if site is None:
            self.append(
                # NOTE: Site name, description, and date created will need to be adjusted
                "INSERT INTO site (slug, name, description, date_created)",
                (
                    site_slug,
                    f"[NEEDS UPDATE] {site_slug}",
                    f"[NEEDS UPDATE] {site_slug}",
                    UNKNOWN_CREATION_DATE,
                ),
            )

            site = Site(slug=site_slug, wikijump_id=self.site_counter.next())
            self.context.add_site(site)

        return site

    def add_text(self, text):
        text_bytes = text.encode("utf-8")
        text_hash = hashlib.sha512(text_bytes).digest()

        if text_hash not in self.text_hashes:
            self.append_sql(
                "INSERT INTO text (hash, contents) VALUES (%s, %s)", (text_hash, text)
            )
            self.text_hashes.add(text_hash)

        return text_hash
