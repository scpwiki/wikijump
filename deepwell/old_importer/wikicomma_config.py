import json
import logging
import re
from collections import namedtuple

WIKIDOT_SITE_REGEX = re.compile(r"https?:\/\/([^\.]+)\.wikidot\.com\/?")

WikicommaConfig = namedtuple("WikicommaConfig", ("sites",))
SiteData = namedtuple("SiteData", ("descr", "slug", "url"))

logger = logging.getLogger(__name__)


def parse_config(path: str) -> WikicommaConfig:
    with open(path) as file:
        data = json.load(file)

    sites = {}
    logger.info("Found sites:")
    for pair in data["wikis"]:
        descr = pair["name"]
        url = pair["url"]

        match = WIKIDOT_SITE_REGEX.match(url)
        if match is None:
            logger.error("Cannot parse site URL: %s", url)
            raise ValueError(url)
        slug = match[1]
        logger.info("* %s ('%s')", slug, descr)

        sites[descr] = SiteData(
            descr=descr,
            slug=slug,
            url=url,
        )

    return WikicommaConfig(sites=sites)
