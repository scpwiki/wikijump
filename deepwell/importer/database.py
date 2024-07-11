import json
import logging
import os
import sqlite3
from typing import Optional

from .wikicomma_config import SiteData
from .utils import kangaroo_twelve, from_js_timestamp

logger = logging.getLogger(__name__)

ANONYMOUS_USER_ID = 2


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

    def add_text(self, cur, contents: str) -> str:
        logger.debug("Adding text entry (len %d)", len(contents))

        hex_hash = kangaroo_twelve(contents)
        cur.execute(
            """
            INSERT INTO text
            (hex_hash, contents)
            VALUES
            (?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (hex_hash, contents),
        )
        return hex_hash

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
                    site_id,
                    site_descr,
                    site_url
                )
                VALUES
                (?, ?, ?, ?)
                ON CONFLICT
                DO UPDATE
                SET
                    site_descr = ?,
                    site_url = ?
                """,
                (
                    slug,
                    id,
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

    def add_page(self, cur, *, site_slug: str, page_descr: str, metadata: dict) -> None:
        logger.info(
            "Inserting into site '%s' page descr '%s'",
            site_slug,
            page_descr,
        )

        page_id = metadata["page_id"]
        sitemap_updated_at = metadata["sitemap_update"] // 1000

        # If a page has been moved, it can leave multiple entries.
        # We want the most recent page if we find such entries.
        result = cur.execute(
            """
            SELECT page_descr, sitemap_updated_at
            FROM page
            WHERE page_id = ?
            AND site_slug = ?
            """,
            (page_id, site_slug),
        ).fetchone()
        if result is not None:
            (prior_page_descr, last_sitemap_updated_at) = result
            if last_sitemap_updated_at < sitemap_updated_at:
                logger.warning(
                    "Found updated version of page ID %d, deleting previous '%s' (%d < %d)",
                    page_id,
                    prior_page_descr,
                    last_sitemap_updated_at,
                    sitemap_updated_at,
                )
                cur.execute(
                    """
                    DELETE FROM page
                    WHERE page_id = ?
                    AND site_slug = ?
                    """,
                    (page_id, site_slug),
                )
                self.add_deleted_page(
                    cur,
                    page_descr=prior_page_descr,
                    site_slug=site_slug,
                    page_id=page_id,
                )
            else:
                logger.warning(
                    "Found another version of page ID %d, looks newer, skipping (%d ≥ %d)",
                    page_id,
                    last_sitemap_updated_at,
                    sitemap_updated_at,
                )
                self.add_deleted_page(
                    cur,
                    page_descr=page_descr,
                    site_slug=site_slug,
                    page_id=page_id,
                )
                return

        # Insert new page
        cur.execute(
            """
            INSERT INTO page
            (
                page_id,
                page_descr,
                page_slug,
                site_slug,
                sitemap_updated_at,
                title,
                locked,
                tags
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                page_id,
                page_descr,
                metadata["name"],
                site_slug,
                sitemap_updated_at,
                metadata.get("title", ""),
                metadata.get("is_locked", False),
                json.dumps(metadata.get("tags", [])),
            ),
        )

    def add_deleted_page(
        self,
        cur,
        *,
        page_descr: str,
        site_slug: str,
        page_id: int,
    ) -> None:
        logger.debug(
            "Adding deleted page: %s / %s (%d)",
            page_descr,
            site_slug,
            page_id,
        )
        cur.execute(
            """
            INSERT INTO page_deleted
            (
                page_descr,
                site_slug,
                page_id
            )
            VALUES
            (?, ?, ?)
            """,
            (page_descr, site_slug, page_id),
        )

    def is_deleted_page(self, *, page_descr: str, site_slug: str) -> bool:
        with self.conn as cur:
            result = cur.execute(
                """
                SELECT *
                FROM page_deleted
                WHERE page_descr = ?
                AND site_slug = ?
                """,
                (page_descr, site_slug),
            ).fetchone()

        exists = result is not None
        logger.debug(
            "Checking if page descr %s exists in site %s: %s",
            page_descr,
            site_slug,
            exists,
        )
        return exists

    def add_page_revision_metadata(self, cur, page_id: int, data: dict) -> None:
        logger.info(
            "Inserting page revision %d for page ID %d",
            data["revision"],
            page_id,
        )

        cur.execute(
            """
            INSERT INTO page_revision
            (
                revision_id,
                revision_number,
                page_id,
                user_id,
                created_at,
                flags,
                comments
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                data["global_revision"],
                data["revision"],
                page_id,
                data["author"] or ANONYMOUS_USER_ID,
                data["stamp"],
                data["flags"],
                data["commentary"],
            ),
        )

    def add_page_revision_wikitext(self, cur, revision_id: int, contents: str) -> None:
        logger.debug("Inserting page revision wikitext for %d", revision_id)

        hex_hash = self.add_text(cur, contents)
        cur.execute(
            """
            INSERT INTO page_revision_wikitext
            (revision_id, wikitext_hash)
            VALUES (?, ?)
            ON CONFLICT
            DO UPDATE
            SET wikitext_hash = ?
            """,
            (
                revision_id,
                hex_hash,
                hex_hash,
            ),
        )

    def add_page_vote(
        self,
        cur,
        *,
        page_id: int,
        user_id: int,
        value: int,
    ) -> None:
        logger.info(
            "Inserting page vote for page ID %d / user ID %d (value %+d)",
            page_id,
            user_id,
            value,
        )

        cur.execute(
            """
            INSERT INTO page_vote
            (
                page_id,
                user_id,
                value
            )
            VALUES
            (?, ?, ?)
            ON CONFLICT
            DO UPDATE
            SET value = ?
            """,
            (
                page_id,
                user_id,
                value,
                value,
            ),
        )

    def add_blob(self, cur, *, hex_hash: str, length: int, mime: str) -> None:
        logger.debug("Inserting blob record")
        cur.execute(
            """
            INSERT INTO blob
            (hex_hash, mime, length)
            VALUES (?, ?, ?)
            """,
            (hex_hash, mime, length),
        )

    def blob_exists(self, hex_hash: str) -> bool:
        with self.conn as cur:
            result = cur.execute(
                """
                SELECT *
                FROM blob
                WHERE hex_hash = ?
                """,
                (hex_hash,),
            ).fetchone()

        exists = result is not None
        logger.debug("Checking blob existence: %s (%s)", hex_hash, exists)
        return exists

    def add_file(
        self,
        cur,
        *,
        file_id: int,
        page_id: int,
        site_slug: str,
        filename: str,
        s3_hash: str,
    ) -> None:
        logger.info("Inserting file for page ID %d", page_id)

        cur.execute(
            """
            INSERT INTO file
            (
                file_id,
                page_id,
                site_slug,
                filename,
                s3_hash
            )
            VALUES
            (?, ?, ?, ?, ?)
            ON CONFLICT
            DO UPDATE
            SET filename = ?,
                s3_hash = ?
            """,
            (
                file_id,
                page_id,
                site_slug,
                filename,
                s3_hash,
                filename,
                s3_hash,
            ),
        )

    def add_forum_category(
        self,
        cur,
        site_slug: str,
        metadata: dict,
    ) -> None:
        forum_category_id = metadata["id"]
        logger.info("Inserting forum category ID %d", forum_category_id)

        cur.execute(
            """
            INSERT INTO forum_category
            (
                forum_category_id,
                site_slug,
                title,
                description,
                last_user_id,
                thread_count,
                post_count,
                full_scan,
                last_page,
                version
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                forum_category_id,
                site_slug,
                metadata["title"],
                metadata["description"],
                metadata.get("lastUser"),
                metadata.get("threads"),
                metadata.get("posts"),
                metadata["full_scan"],
                metadata["last_page"],
                metadata["version"],
            ),
        )

    def add_forum_thread(self, cur, forum_category_id: int, metadata: dict) -> None:
        forum_thread_id = metadata["id"]
        logger.info("Inserting forum thread ID %d", forum_thread_id)

        cur.execute(
            """
            INSERT INTO forum_thread
            (
                forum_thread_id,
                forum_category_id,
                title,
                description,
                created_at,
                created_by,
                post_count,
                sticky,
                locked,
                version
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                forum_thread_id,
                forum_category_id,
                metadata["title"],
                metadata["description"],
                metadata["started"],
                metadata["startedUser"],
                metadata["postsNum"],
                metadata["sticky"],
                metadata.get("isLocked", False),
                metadata.get("version"),
            ),
        )

    def add_forum_post(
        self,
        cur,
        *,
        forum_thread_id: int,
        parent_post_id: Optional[int],
        metadata: dict,
    ) -> None:
        forum_post_id = metadata["id"]
        logger.info("Inserting forum post ID %d", forum_post_id)

        cur.execute(
            """
            INSERT INTO forum_post
            (
                forum_post_id,
                forum_thread_id,
                parent_post_id,
                title,
                created_at,
                created_by,
                edited_at,
                edited_by
            )
            VALUES
            (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                forum_post_id,
                forum_thread_id,
                parent_post_id,
                metadata["title"],
                metadata["stamp"],
                metadata["poster"],
                metadata.get("lastEdit"),
                metadata.get("lastEditBy"),
            ),
        )

    def add_forum_post_revision(self, cur, post_id: int, metadata: dict) -> None:
        revision_id = metadata["id"]
        logger.info("Inserting forum post ID %d (revision ID %d)", post_id, revision_id)

        cur.execute(
            """
            INSERT INTO forum_post_revision
            (
                forum_post_revision_id,
                forum_post_id,
                title,
                created_at,
                created_by
            )
            VALUES
            (?, ?, ?, ?, ?)
            ON CONFLICT
            DO NOTHING
            """,
            (
                revision_id,
                post_id,
                metadata["title"],
                metadata["stamp"],
                metadata["author"],
            ),
        )

    def add_forum_post_wikitext(self, cur, forum_post_id: int, contents: str):
        logger.info("Inserting latest forum post wikitext for ID %d", forum_post_id)
        hex_hash = self.add_text(cur, contents)

        cur.execute(
            """
            INSERT INTO forum_post_wikitext
            (
                forum_post_id,
                wikitext_hash
            )
            VALUES
            (?, ?)
            ON CONFLICT
            DO UPDATE
            SET wikitext_hash = ?
            """,
            (forum_post_id, hex_hash, hex_hash),
        )

    def add_forum_post_revision_wikitext(
        self,
        cur,
        forum_post_revision_id: int,
        contents: str,
    ):
        logger.info(
            "Inserting forum post revision wikitext for ID %d",
            forum_post_revision_id,
        )
        hex_hash = self.add_text(cur, contents)

        cur.execute(
            """
            INSERT INTO forum_post_revision_wikitext
            (
                forum_post_revision_id,
                wikitext_hash
            )
            VALUES
            (?, ?)
            ON CONFLICT
            DO UPDATE
            SET wikitext_hash = ?
            """,
            (forum_post_revision_id, hex_hash, hex_hash),
        )
