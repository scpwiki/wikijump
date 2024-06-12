import hashlib

from .database import Database

import boto3


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
        logger,
        wikicomma_directory,
        sqlite_path,
        aws_profile,
        s3_bucket,
    ):
        self.logger = logger
        self.wikicomma_directory = wikicomma_directory
        self.database = Database(sqlite_path)
        self.aws_profile = aws_profile
        self.boto_session = boto3.Session(profile_name=aws_profile)
        self.s3_client = self.boto_session.client("s3")
        self.s3_bucket = s3_bucket

    def s3_object_exists(self, s3_path):
        try:
            self.s3_client.head_object(
                Bucket=self.s3_bucket,
                Key=s3_path,
            )
            return True
        except:
            return False

    def upload_file(self, file_path):
        with open(path, "rb") as file:
            data = file.read()
            s3_path = hashlib.sha256(data).hexdigest()

        if not data:
            self.logger.debug("Skipping upload of empty S3 object")
        elif self.s3_object_exists(s3_path):
            self.logger.debug("S3 object %s already exists", s3_path)
        else:
            self.logger.info("Uploading S3 object %s (len %d)", s3_path, len(data))
            self.s3_client.upload_file(
                Bucket=self.s3_bucket,
                Key=s3_path,
                Body=data,
                ContentLength=len(data),
            )

    def run(self):
        ...

    def close(self):
        self.database.close()
