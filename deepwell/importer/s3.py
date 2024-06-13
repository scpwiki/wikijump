import logging

import boto3

logger = logging.getLogger(__name__)


class S3:
    __slots__ = (
        "aws_profile",
        "session",
        "client",
        "bucket",
    )

    def __init__(self, *, aws_profile, bucket) -> None:
        self.aws_profile = aws_profile
        self.session = boto3.Session(profile_name=aws_profile)
        self.client = self.session.client("s3")
        self.bucket = bucket

    def exists(self, s3_path: str) -> bool:
        try:
            self.s3_client.head_object(
                Bucket=self.s3_bucket,
                Key=s3_path,
            )
            return True
        except:
            return False

    def upload(self, file_path: str) -> None:
        with open(path, "rb") as file:
            data = file.read()
            s3_path = hashlib.sha256(data).hexdigest()

        if not data:
            logger.debug("Skipping upload of empty S3 object")
        elif self.exists(s3_path):
            logger.debug("S3 object %s already exists", s3_path)
        else:
            logger.info("Uploading S3 object %s (len %d)", s3_path, len(data))
            self.client.upload_file(
                Bucket=self.s3_bucket,
                Key=s3_path,
                Body=data,
                ContentLength=len(data),
            )
