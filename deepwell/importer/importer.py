import glob
import hashlib
import json
import logging
import os

from .database import Database

import boto3

logger = logging.getLogger("importer")


class Importer:
    __slots__ = (
        "logger",
        "wikicomma_directory",
        "database",
        "aws_profile",
        "boto_session",
        "s3_client",
        "s3_bucket",
    )

    def __init__(
        self,
        *,
        wikicomma_directory,
        sqlite_path,
        delete_sqlite,
        aws_profile,
        s3_bucket,
    ) -> None:
        self.wikicomma_directory = wikicomma_directory
        self.database = Database(sqlite_path, delete=delete_sqlite)
        self.aws_profile = aws_profile
        self.boto_session = boto3.Session(profile_name=aws_profile)
        self.s3_client = self.boto_session.client("s3")
        self.s3_bucket = s3_bucket

    def s3_object_exists(self, s3_path: str) -> bool:
        try:
            self.s3_client.head_object(
                Bucket=self.s3_bucket,
                Key=s3_path,
            )
            return True
        except:
            return False

    def upload_file(self, file_path: str) -> None:
        with open(path, "rb") as file:
            data = file.read()
            s3_path = hashlib.sha256(data).hexdigest()

        if not data:
            logger.debug("Skipping upload of empty S3 object")
        elif self.s3_object_exists(s3_path):
            logger.debug("S3 object %s already exists", s3_path)
        else:
            logger.info("Uploading S3 object %s (len %d)", s3_path, len(data))
            self.s3_client.upload_file(
                Bucket=self.s3_bucket,
                Key=s3_path,
                Body=data,
                ContentLength=len(data),
            )

    def data_dir(self, subdirectory: str) -> str:
        return os.path.join(self.wikicomma_directory, subdirectory)

    def run(self) -> None:
        logger.info("Starting Wikicomma importer...")

        self.database.seed()
        self.process_users()
        ...

    def close(self) -> None:
        self.database.close()

    def process_users(self) -> None:
        logger.info("Processing users...")

        directory = self.data_dir("_users")
        for path in glob.iglob(f"{directory}/*.json"):
            logger.debug("Reading %s", path)
            with open(path) as file:
                data = json.load(file)

            filename = os.path.basename(path)
            if filename == "pending.json":
                logger.debug("Skipping pending user list")
                continue

            self.database.add_user_block(data, filename)
