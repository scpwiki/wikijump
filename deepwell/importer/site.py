import json
import logging
import os
import re
from functools import cache
from typing import Tuple, Union
from urllib.request import urlopen

from .database import Database

SITE_ID_REGEX = re.compile(r"WIKIREQUEST\.info\.siteId = (\d+);")

logger = logging.getLogger(__name__)


class SiteImporter:
    __slots__ = (
        "directory",
        "database",
        "site_descr",
        "site_slug",
        "site_url",
        "site_id",
        "page_ids",
    )

    def __init__(
        self,
        *,
        directory: str,
        database: Database,
        site_descr: str,
        site_slug: str,
        site_url: str,
    ) -> None:
        self.directory = directory
        self.database = database
        self.site_descr = site_descr
        self.site_slug = site_slug
        self.site_url = site_url
        self.site_id = self.get_site_id(site_url)
        self.page_ids = {}

    @staticmethod
    def convert_page_slug(page_slug: str) -> str:
        if page_slug.startswith("_"):
            # a _default category page that starts with an underscore, e.g. _template
            return page_slug

        # replace only the first underscore
        # the second (if present) is a special page, like _404
        converted, _ = re.subn("_", ":", page_slug, 1)
        return converted

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

    def get_page_id(self, page_slug: str) -> int:
        page_id = self.page_ids.get(page_slug)
        if page_id is not None:
            return page_id

        with self.database.conn as cur:
            result = cur.execute(
                """
                SELECT page_id FROM page
                WHERE page_slug = ?
                AND site_slug = ?
                """,
                (page_slug, self.site_slug),
            ).fetchone()

        if result is not None:
            (page_id,) = result
            self.page_ids[page_slug] = page_id
            return page_id

        raise RuntimeError(f"Cannot find page ID for page '{page_slug}' in site '{self.site_slug}'")

    def file_dir(self) -> str:
        return os.path.join(self.directory, "files")

    def forum_dir(self) -> str:
        return os.path.join(self.directory, "forum")

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
        # TODO
        ...

    def process_pages(self) -> None:
        self.process_page_ids()
        self.process_page_metadata()
        # TODO
        ...

    def process_page_ids(self) -> None:
        logger.info("Ingesting page ID mappings for site %s", self.site_slug)
        mapping = self.json(self.meta_path("page_id_map.json"))
        with self.database.conn as cur:
            for id_str, page_slug in mapping.items():
                logger.debug("Found page '%s' (%s)", page_slug, id_str)
                id = int(id_str)
                self.page_ids[page_slug] = id
                self.database.add_page(
                    cur,
                    site_slug=self.site_slug,
                    page_slug=page_slug,
                    page_id=id,
                )

    def process_page_metadata(self) -> None:
        logger.info("Ingesting page revision metadata for site %s", self.site_slug)
        meta_directory = self.meta_path("pages")
        for path in os.listdir(meta_directory):
            logger.debug("Processing page metadata %s", path)

            page_slug, ext = os.path.splitext(path)
            assert ext == ".json", "Extension for page metadata not JSON"
            path = os.path.join(meta_directory, path)

            page_slug = self.convert_page_slug(page_slug)
            page_id = self.get_page_id(page_slug)

            metadata = self.json(path)
            assert metadata["page_id"] == page_id
            assert metadata["name"] == page_slug

            with self.database.conn as cur:
                self.database.add_page_metadata(
                    cur,
                    page_id,
                    metadata,
                )
                self.process_page_revisions(cur, page_id, metadata["revisions"])
                self.process_page_votes(cur, page_id, metadata["votings"])

    def process_page_revisions(self, cur, page_id: int, revisions: list[dict]) -> None:
        logger.debug("Ingesting page revision metadata for page ID %d", page_id)
        for revision in revisions:
            self.database.add_page_revision_metadata(cur, page_id, revision)

    def process_page_votes(self, cur, page_id: int, votes: list[Tuple[int, int]]) -> None:
        logger.debug("Ingesting page votes for page ID %d", page_id)
        for user_id, bool_value in votes:
            int_value = 1 if bool_value else -1
            self.database.add_page_vote(cur, user_id=user_id, page_id=page_id, value=int_value)

    def process_files(self) -> None:
        # TODO
        ...

    def process_forum(self) -> None:
        # TODO
        ...
