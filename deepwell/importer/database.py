import os
import logging
import sqlite3

from .wikicomma_config import SiteData
from .utils import from_js_timestamp

logger = logging.getLogger(__name__)


class Database:
    __slots__ = ("conn",)

    def __init__(self, db_url: str, delete: bool = False) -> None:
        if delete:
            if os.path.exists(db_url):
                logger.debug("Deleting previous SQLite at %s", db_url)
                os.remove(db_url)

        self.conn = sqlite3.connect(db_url)

    def seed(self) -> None:
        seed_path = os.path.join(os.path.dirname(__file__), "seed.sql")

        with open(seed_path) as file:
            self.conn.executescript(file.read())

    def close(self) -> None:
        self.conn.close()

    def add_user_block(self, block: dict, filename: str) -> None:
        logger.info("Found %d users in block '%s'", len(block), filename)

        with self.conn as cur:
            # key is redundant, string of user ID
            for data in block.values():
                self.add_user(cur, data)

    def add_site(self, *, slug: str, descr: str, url: str, id: int) -> None:
        logger.info(
            "Inserting site '%s' (%s, %d)",
            descr,
            slug,
            id,
        )

        with self.conn as cur:
            cur.execute(
                """
                INSERT INTO site
                (
                    site_slug,
                    site_descr,
                    site_url,
                )
                VALUES
                (?, ?, ?)
                ON CONFLICT
                DO UPDATE
                SET
                    site_descr = ?,
                    site_url = ?
                """,
                (
                    slug,
                    descr,
                    url,
                    descr,
                    url,
                ),
            )

    def add_user(self, cur, data: dict) -> None:
        logger.info(
            "Inserting user '%s' (%s, %d)",
            data["full_name"],
            data["username"],
            data["user_id"],
        )

        cur.execute(
            """
            INSERT INTO user
            (
                user_slug,
                user_name,
                user_id,
                user_since,
                account_type,
                karma,
                fetched_at,
                real_name,
                gender,
                birthday,
                location,
                website
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                data["username"],  # slug (e.g. foo-bar)
                data["full_name"],  # name (e.g. Foo Bar)
                data["user_id"],
                data["wikidot_user_since"],
                data["account_type"],
                data["activity"],
                data["fetched_at"] // 1000,
                data.get("real_name"),
                data.get("gender"),
                from_js_timestamp(data.get("birthday")),
                data.get("location"),
                data.get("website"),
            ),
        )

    def add_page(self, cur, *, page_id: int, site_slug: str, page_slug: str) -> None:
        logger.info("Inserting page '%s' (%d)", page_slug, page_id)

        cur.execute(
            """
            INSERT INTO page
            (
                page_id,
                site_slug,
                page_slug
            )
            VALUES
            (?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                page_id,
                site_slug,
                page_slug,
            ),
        )
