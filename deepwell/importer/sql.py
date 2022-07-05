import hashlib

from .constants import UNKNOWN_CREATION_DATE
from .counter import IncrementingCounter
from .structures import Page, Site, User

import psycopg2


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

        self.users = {}
        self.pages = {}
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

    def add_user(self, user_id, user_slug, name):
        user = self.users.get(user_id)
        if user is None:
            # TODO change over when user table changes, remaining fields
            self.append(
                "INSERT INTO users (id, slug, username) VALUES (%s, %s, %s)",
                (user_id, user_slug, name),
            )

            user = User(slug=user_slug, wikidot_id=user_id)
            self.users[user_id] = user
        return user


    def add_site(self, site_slug):
        site = self.sites.get(site_slug)
        if site is None:
            self.append(
                # NOTE: Site name, description, and date created will need to be adjusted
                "INSERT INTO site (slug, name, description, date_created) VALUES (%s, %s, %s)",
                (
                    site_slug,
                    f"[NEEDS UPDATE] {site_slug}",
                    f"[NEEDS UPDATE] {site_slug}",
                    UNKNOWN_CREATION_DATE,
                ),
            )

            site = Site(slug=site_slug, wikijump_id=self.site_counter.next())
            self.sites[site_slug] = site
        return site

    def add_page(self, site, page_id, page_slug, created_at, updated_at, page_category_id, discussion_thread_id):
        page = self.pages.get(page_id)
        if page is None:
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
            self.pages[page_id] = page
        return page

    def add_text(self, text):
        text_bytes = text.encode("utf-8")
        text_hash = hashlib.sha512(text_bytes).digest()

        if text_hash not in self.text_hashes:
            self.append_sql(
                "INSERT INTO text (hash, contents) VALUES (%s, %s)", (text_hash, text)
            )
            self.text_hashes.add(text_hash)

        return text_hash
