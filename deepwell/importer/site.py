import logging
import re
from typing import Union
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

    @staticmethod
    def get_site_id(site_url: str) -> int:
        logger.info("Downloading web page %s to scrape site ID", site_url)

        with urlopen(site_url) as file:
            html = file.read().decode("utf-8")

        match = SITE_ID_REGEX.search(html)
        if match is None:
            logger.error("Unable to find site ID in HTML")
            raise ValueError(site_url)

        return int(match[1])

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
        # TODO
        ...

    def process_page_ids(self) -> None:
        logger.info("Ingesting page ID mappings for site %s", self.site_slug)
        mapping = self.json(self.meta_path("page_id_map.json"))
        with self.database.conn as cur:
            for id_str, page_slug in mapping.items():
                logger.debug("Found page '%s' (%d)", page_slug, id_str)
                id = int(id_str)
                self.database.add_page(
                    cur,
                    site_slug=self.site_slug,
                    page_slug=page_slug,
                    page_id=id,
                )

    def process_files(self) -> None:
        # TODO
        ...

    def process_forum(self) -> None:
        # TODO
        ...
