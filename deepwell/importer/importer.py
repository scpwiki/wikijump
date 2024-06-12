import boto3

class Importer:
    __slots__ = (
        "logger",
        "wikicomma_directory",
        "sqlite_path",
        "aws_profile",
        "boto_session",
        "s3_client",
    )

    def __init__(self, *, logger, wikicomma_directory, sqlite_path, aws_profile):
        self.logger = logger
        self.wikicomma_directory = wikicomma_directory
        self.sqlite_path = sqlite_path
        self.aws_profile = aws_profile
        self.boto_session = boto3.Session(profile_name=aws_profile)
        self.s3_client = self.boto_session.client("s3")

    def run(self):
        ...
