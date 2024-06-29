import json
import logging
import os
import re
from functools import cache
from io import BytesIO
from typing import Tuple, Union
from urllib.parse import unquote as percent_unquote
from urllib.request import urlopen

import py7zr

from .database import Database
from .s3 import S3

SITE_ID_REGEX = re.compile(r"WIKIREQUEST\.info\.siteId = (\d+);")

logger = logging.getLogger(__name__)


class SiteImporter:
    __slots__ = (
        "directory",
        "database",
        "s3",
        "site_descr",
        "site_slug",
        "site_url",
        "site_id",
    )

    def __init__(
        self,
        *,
        directory: str,
        database: Database,
        s3: S3,
        site_descr: str,
        site_slug: str,
        site_url: str,
    ) -> None:
        self.directory = directory
        self.database = database
        self.s3 = s3
        self.site_descr = site_descr
        self.site_slug = site_slug
        self.site_url = site_url
        self.site_id = self.get_site_id(site_url)

    @cache
    def get_site_id(self, site_url: str) -> int:
        with self.database.conn as cur:
            result = cur.execute(
                """
                SELECT site_id FROM site
                WHERE site_url = ?
                """,
                (site_url,),
            ).fetchone()

        if result is not None:
            site_id = result[0]
            logger.debug("Found site ID for URL %s: %d", site_url, site_id)
            return site_id

        logger.info("Downloading web page %s to scrape site ID", site_url)

        with urlopen(site_url) as file:
            html = file.read().decode("utf-8")

        match = SITE_ID_REGEX.search(html)
        if match is None:
            logger.error("Unable to find site ID in HTML")
            raise ValueError(site_url)

        return int(match[1])

    def get_page_id(self, *, page_slug: str = None, page_descr: str = None) -> int:
        with self.database.conn as cur:
            match bool(page_slug), bool(page_descr):
                case True, False:
                    query = """
                    SELECT page_id
                    FROM page
                    WHERE page_slug = ?
                        AND site_slug = ?
                    """
                    parameters = (page_slug, self.site_slug)
                case False, True:
                    query = """
                    SELECT page_id
                    FROM page
                    WHERE page_descr = ?
                        AND site_slug = ?
                    """
                    parameters = (page_descr, self.site_slug)
                case _, _:
                    raise ValueError(
                        "Must pass exactly one parameter into get_page_id()",
                    )

            result = cur.execute(query, parameters).fetchone()

        if result is None:
            raise RuntimeError(
                f"Cannot find page ID for page_descr={page_descr} / page_slug={page_slug} in site '{self.site_slug}'",
            )

        (page_id,) = result
        return page_id

    def get_page_descr(self, page_id: int) -> str:
        with self.database.conn as cur:
            result = cur.execute(
                """
                SELECT page_metadata.page_descr
                FROM page
                JOIN page_metadata
                    ON page.page_id = page_metadata.page_id
                WHERE page_metadata.page_id = ?
                    AND page.site_slug = ?
                """,
                (page_id, self.site_slug),
            ).fetchone()

        if result is None:
            raise RuntimeError(
                f"Cannot find page descr for page ID {page_id} in site '{self.site_slug}'",
            )

        (page_descr,) = result
        return page_descr

    def get_revision_id(self, cur, page_id: int, revision_number: int) -> int:
        result = cur.execute(
            """
            SELECT revision_id
            FROM page_revision
            WHERE page_id = ?
            AND revision_number = ?
            """,
            (page_id, revision_number),
        ).fetchone()
        if result is None:
            raise RuntimeError(
                f"Cannot find page revision for (page {page_id}, rev {revision_number})",
            )
        (revision_id,) = result
        return revision_id

    @property
    def file_dir(self) -> str:
        return os.path.join(self.directory, "files")

    @property
    def forum_dir(self) -> str:
        return os.path.join(self.directory, "forum")

    @property
    def page_dir(self) -> str:
        return os.path.join(self.directory, "pages")

    def meta_path(self, path: str) -> str:
        return os.path.join(self.directory, "meta", path)

    def json(self, path: str) -> Union[list, dict]:
        with open(path) as file:
            return json.load(file)

    def run(self) -> None:
        self.database.add_site(
            slug=self.site_slug,
            descr=self.site_descr,
            url=self.site_url,
            id=self.site_id,
        )
        self.process_pages()
        self.process_files()
        self.process_forum()

    def process_pages(self) -> None:
        self.process_page_metadata()
        self.process_page_wikitext()

    def process_page_metadata(self) -> None:
        logger.info("Ingesting page revision metadata for site %s", self.site_slug)
        meta_directory = self.meta_path("pages")
        for path in os.listdir(meta_directory):
            logger.debug("Processing page metadata from '%s'", path)

            # NOTE: Usually page_slug is the same as page_descr, but if
            #       there are any colons in it, then they don't match.
            #       So we can use it as a temporary unique identifier
            #       but *not* as the slug.
            page_descr, ext = os.path.splitext(path)
            assert ext == ".json", "Extension for page metadata not JSON"
            path = os.path.join(meta_directory, path)

            metadata = self.json(path)
            with self.database.conn as cur:
                self.database.add_page(
                    cur,
                    site_slug=self.site_slug,
                    page_descr=page_descr,
                    metadata=metadata,
                )
                page_id = metadata["page_id"]
                self.process_page_revisions_metadata(
                    cur,
                    page_id,
                    metadata["revisions"],
                )
                self.process_page_votes(cur, page_id, metadata["votings"])

    def process_page_revisions_metadata(
        self,
        cur,
        page_id: int,
        revisions: list[dict],
    ) -> None:
        logger.debug("Ingesting page revision metadata for page ID %d", page_id)
        for revision in revisions:
            self.database.add_page_revision_metadata(cur, page_id, revision)

    def process_page_votes(
        self,
        cur,
        page_id: int,
        votes: list[Tuple[int, int]],
    ) -> None:
        logger.debug("Ingesting page votes for page ID %d", page_id)
        for user_id, bool_value in votes:
            int_value = 1 if bool_value else -1
            self.database.add_page_vote(
                cur,
                user_id=user_id,
                page_id=page_id,
                value=int_value,
            )

    def process_page_wikitext(self) -> None:
        logger.info("Ingesting page wikitext for site %s", self.site_slug)
        for path in os.listdir(self.page_dir):
            logger.debug("Processing page wikitext from '%s'", path)

            # See above note on page_descr
            page_descr, ext = os.path.splitext(path)
            assert ext == ".7z", "Extension for page wikitexts not 7z"
            path = os.path.join(self.page_dir, path)

            # Extract page sources for each revision
            with py7zr.SevenZipFile(path, "r") as archive:
                sources = archive.readall()

            page_id = self.get_page_id(page_descr=page_descr)
            # Convert and begin adding to the database
            self.process_page_revisions_wikitext(page_id, sources)

    def process_page_revisions_wikitext(
        self,
        page_id: int,
        sources: dict[str, BytesIO],
    ) -> None:
        logger.debug("Ingesting %d page revision wikitexts", len(sources))

        with self.database.conn as cur:
            for filename, buf in sources.items():
                # Get revision number from filename
                revision_number_str, ext = os.path.splitext(filename)
                assert ext == ".txt", "Extension for page revision wikitext not txt"
                revision_number = int(revision_number_str)
                logger.info("Ingesting page revision %d (%d)", page_id, revision_number)

                # Get revision ID
                revision_id = self.get_revision_id(cur, page_id, revision_number)

                # Converting from binary, mostly to ensure it's UTF-8
                contents = buf.read().decode("utf-8")

                # Run ingestion for this revision
                self.database.add_page_revision_wikitext(cur, revision_id, contents)

    def process_files(self) -> None:
        logger.info("Ingesting files for site %s", self.site_slug)

        mapping = self.json(self.meta_path("file_map.json"))
        with self.database.conn as cur:
            for file_id, entry in mapping.items():
                file_id = int(file_id)
                wikidot_url = entry["url"]
                logger.debug("Processing file stored at %s", wikidot_url)

                page_slug_url, filename = os.path.split(entry["path"])
                page_slug = percent_unquote(page_slug_url)
                page_id = self.get_page_id(page_slug=page_slug)

                path = os.path.join(self.file_dir, page_slug_url, str(file_id))
                s3_hash = self.s3.upload(path)

                self.database.add_file(
                    cur,
                    file_id=file_id,
                    page_id=page_id,
                    site_slug=self.site_slug,
                    filename=filename,
                    s3_hash=s3_hash,
                )

        # TODO
        ...

    def process_forum(self) -> None:
        logger.info("Ingesting forum data for site %s", self.site_slug)
        # TODO
        ...
