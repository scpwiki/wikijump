import logging
import re
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
        self.site_descr = site_descr
        self.site_slug = site_slug
        self.site_url = site_url
        self.site_id = self.get_site_id(site_url)

    @staticmethod
    def get_site_id(site_url: str) -> int:
        logger.info("Downloading web page %s to scrape site ID", site_url)

        with urlopen(site_url) as file:
            html = file.read().decode("utf-8")

        match = SITE_ID_REGEX.find(html)
        if match is None:
            logger.error("Unable to find site ID in HTML")
            raise ValueError(site_url)

        return int(match[1])

    def run(self) -> None:
        self.database.add_site(
            slug=self.site_slug,
            descr=self.site_descr,
            url=self.site_url,
            id=self.site_id,
        )
        ...
