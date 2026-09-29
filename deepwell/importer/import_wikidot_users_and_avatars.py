#!/usr/bin/env python

"""
Import script to read in a wikidot_users.csv file and associated avatars.
"""

import argparse
import csv
import lzma
import os
import sqlite3
from datetime import date, datetime, timezone
from types import TracebackType
from typing import Self, TextIO

from deepwell_common import (
    Deepwell,
    ImportDeletedUser,
    ImportExistingUser,
    ImportUserData,
    UploadBlobData,
)
from log_common import setup_logging


def parse_boolean(value: str) -> bool:
    match value:
        case "true":
            return True
        case "false":
            return False
        case _:
            raise ValueError(value)


def parse_karma(value: str) -> int:
    match value:
        case "none":
            return 0
        case "low":
            return 1
        case "medium":
            return 2
        case "high":
            return 3
        case "very high":
            return 4
        case "guru":
            return 5
        case _:
            raise ValueError(value)


def empty_str_as_none(value: str) -> str | None:
    match value:
        case "":
            return None
        case _:
            return value


def open_csv_file(path: str) -> TextIO:
    if path.casefold().endswith(".xz"):
        return lzma.open(path, mode="rt")
    else:
        return open(path)


class AvatarReader:
    def __init__(self, directory: str) -> None:
        self.main_directory = directory
        self.files_directory = os.path.join(directory, "files")
        self.avatar_db_path = os.path.join(directory, "avatars.db")
        self.avatar_db_conn: sqlite3.Connection | None = None

    def __enter__(self) -> Self:
        self.avatar_db_conn = sqlite3.connect(self.avatar_db_path)
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_value: BaseException | None,
        traceback: TracebackType | None,
    ) -> bool | None:
        assert self.avatar_db_conn, "no active connection"
        self.avatar_db_conn.close()
        return None

    def get(self, user_id: int) -> tuple[bytes, str] | None:
        # Get avatar hash
        assert self.avatar_db_conn, "no active connection"
        results = self.avatar_db_conn.execute(
            "SELECT avatar_md5_hash FROM avatars WHERE user_id = ?",
            [user_id],
        )
        (avatar_md5_hash,) = results.fetchone()

        if avatar_md5_hash is None:
            return None

        # Fetch its data from the filesystem
        avatar_path = os.path.join(self.files_directory, avatar_md5_hash)
        content_type_path = f"{avatar_path}.content-type"

        with open(avatar_path, "rb") as file:
            blob = file.read()

        with open(content_type_path, "r") as file:
            content_type = file.read().rstrip()

        return blob, content_type


if __name__ == "__main__":
    argparser = argparse.ArgumentParser("import_wikidot_users_and_avatars")
    argparser.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="Enable debug logging",
    )
    argparser.add_argument(
        "-H",
        "--host",
        default="localhost",
        help="DEEPWELL host to send import requests to",
    )
    argparser.add_argument(
        "-p",
        "--port",
        default=2747,
        type=int,
        help="DEEPWELL port on the host",
    )
    argparser.add_argument(
        # :'(
        "--users-fetched-date",
        default=datetime(2026, 1, 21, 0, 0, 0, tzinfo=timezone.utc),
        type=datetime.fromisoformat,
        help="The date to record the users as having been fetched at",
    )
    argparser.add_argument(
        "--importer-user-id",
        default=-2,
        type=int,
        help="The user ID to associate the import operations with",
    )
    argparser.add_argument(
        "--importer-ip-address",
        default="127.0.0.255",  # localhost, but distinct for logging purposes
        help="The IP address to associate the import operations with",
    )
    argparser.add_argument(
        "csv_file",
        help="wikidot_users.csv input file",
    )
    argparser.add_argument(
        "avatars_directory",
        help="Fetched avatars directory, assumes avatars.db and files/ exist inside",
    )
    args = argparser.parse_args()

    logger = setup_logging(args.debug)
    logger.info("Running import_wikidot_users_and_avatars with configuration:")
    logger.info("* DEEPWELL server:        %s %d", args.host, args.port)
    logger.info("* Avatar directory:       %s", args.avatars_directory)
    logger.info("* wikidot_users.csv file: %s", args.csv_file)

    deepwell = Deepwell(args.host, args.port)
    with (
        AvatarReader(args.avatars_directory) as avatars,
        open_csv_file(args.csv_file) as file,
    ):
        reader = csv.reader(file)

        # Skip the header
        _ = next(reader)

        # Each subsequent row is a user
        for row in reader:
            (
                user_id_raw,
                created_at_raw,
                deleted_raw,
                user_name_raw,
                user_slug_raw,
                real_name_raw,
                gender_raw,
                birthday_raw,
                location_raw,
                about_raw,
                website_raw,
                account_type,
                karma_level_raw,
            ) = row

            # Transform fields
            user_id = int(user_id_raw)
            created_at_naive = datetime.fromisoformat(created_at_raw)
            created_at = created_at_naive.replace(tzinfo=timezone.utc)
            deleted = parse_boolean(deleted_raw)
            user_name = empty_str_as_none(user_name_raw)
            user_slug = empty_str_as_none(user_slug_raw)
            real_name = empty_str_as_none(real_name_raw)
            gender = empty_str_as_none(gender_raw)
            birthday = date.fromisoformat(birthday_raw) if birthday_raw else None
            location = empty_str_as_none(location_raw)
            about = empty_str_as_none(about_raw)
            website = empty_str_as_none(website_raw)
            karma_level = parse_karma(karma_level_raw)
            is_pro = account_type == "Pro"

            # Check if there's an avatar for this user
            avatar = avatars.get(user_id)
            if avatar is None:
                # nothing to upload, default avatar
                blob_id = None
            else:
                # upload avatar
                blob, content_type = avatar
                blob_request = UploadBlobData(
                    uploading_user_id=args.importer_user_id,
                    blob=blob,
                    mime_type=content_type,
                )
                blob_id = deepwell.upload_blob(blob_request)

            # Build import request
            user_type: ImportExistingUser | ImportDeletedUser
            if deleted:
                user_type = ImportDeletedUser()
            else:
                assert user_name is not None
                assert user_slug is not None
                user_type = ImportExistingUser(
                    name=user_name,
                    slug=user_slug,
                )

            import_request = ImportUserData(
                user_id=user_id,
                created_at=created_at,
                fetched_at=args.users_fetched_date,  # not in wikidot_users.csv :(
                wikidot_user_type=user_type,
                avatar_uploaded_blob_id=blob_id,
                real_name=real_name,
                gender=gender,
                birthday=birthday,
                location=location,
                biography=about,
                website=website,
                karma=karma_level,
                is_pro=is_pro,
                importing_user_id=args.importer_user_id,
                ip_address=args.importer_ip_address,
            )
            deepwell.import_user(import_request)
