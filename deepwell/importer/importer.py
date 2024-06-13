import hashlib
import json
import logging
import os

from .database import Database
from .s3 import S3

logger = logging.getLogger(__name__)


class Importer:
    __slots__ = (
        "logger",
        "wikicomma_config",
        "wikicomma_directory",
        "database",
        "s3",
    )

    def __init__(
        self,
        *,
        wikicomma_config,
        wikicomma_directory,
        sqlite_path,
        delete_sqlite,
        aws_profile,
        s3_bucket,
    ) -> None:
        self.wikicomma_config = wikicomma_config
        self.wikicomma_directory = wikicomma_directory
        self.database = Database(sqlite_path, delete=delete_sqlite)
        self.s3 = S3(aws_profile=aws_profile, bucket=s3_bucket)

    def run(self) -> None:
        logger.info("Starting Wikicomma importer...")

        self.database.seed()
        self.process_users()
        self.process_sites()

    def close(self) -> None:
        self.database.close()

    def process_users(self) -> None:
        logger.info("Processing users...")

        directory = os.path.join(self.wikicomma_directory, "_users")
        for filename in os.listdir(directory):
            if filename == "pending.json":
                logger.debug("Skipping pending user list")
                continue

            path = os.path.join(directory, filename)
            logger.debug("Reading %s", path)
            with open(path) as file:
                data = json.load(file)

            self.database.add_user_block(data, filename)

    def process_sites(self) -> None:
        logger.info("Processing sites...")

        for site_descr in os.listdir(self.wikicomma_directory):
            if site_descr == "_users":
                logger.debug("Skipping user list")
                continue

            # NOTE: site_descr != site_slug
            self.process_site(site_descr)

    def process_site(self, site_descr: str) -> None:
        logger.info("Processing site '%s'...", site_descr)
        directory = os.path.join(self.wikicomma_directory, site_descr)

        site_data = self.wikicomma_config.sites[site_descr]
        self.database.add_site(site_data)
