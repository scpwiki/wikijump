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
from typing import TextIO

from deepwell_common import Deepwell, ImportUserData, ImportExistingUser, ImportDeletedUser


def boolean_from_str(value: str) -> bool:
    match value:
        case "true":
            return True
        case "false":
            return False
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
        self.avatar_db_path = os.path.join(directory, "avatars.db")
        self.avatar_db_conn = None
        self.files_directory = os.path.join(directory, "files")

    def __enter__(self) -> "AvatarReader":
        self.avatar_db_conn = sqlite3.connect(self.avatar_db_path)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.avatar_db_conn.close()

    def get_avatar(self, user_id: int) -> tuple[bytes, str] | None:
        # Get avatar hash
        results = self.avatar_db_conn.execute("SELECT avatar_md5_hash FROM avatars WHERE user_id = ?", [user_id])
        (avatar_md5_hash,) = results.fetchone()

        if avatar_md5_hash is None:
            return None

        # Fetch its data from the filesystem
        avatar_path = os.path.join(self.files_directory, avatar_md5_hash)
        content_type_path = f"{avatar_path}.content-type"

        with open(avatar_path, "rb") as file:
            blob = file.read()

        with open(content_type_path, "r") as file:
            content_type = file.read()

        return blob, content_type


if __name__ == "__main__":
    argparser = argparse.ArgumentParser("import_wikidot_users_and_avatars")
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
        "csv_file",
        help="wikidot_users.csv input file",
    )
    argparser.add_argument(
        "avatars_directory",
        help="Fetched avatars directory, assumes avatars.db and files/ exist inside",
    )
    args = argparser.parse_args()

    deepwell = Deepwell(args.host, args.port)
    with open_csv_file(args.csv_file) as file:
        reader = csv.reader(file)

        # Skip the header
        _ = next(reader)

        # Each subsequent row is a user
        for row in reader:
            (
                user_id_raw,
                created_at_raw,
                deleted_raw,
                user_name,
                user_slug,
                real_name,
                gender,
                birthday_raw,
                location,
                about,
                website,
                account_type,
                karma_level_raw,
            ) = row

            # Transform fields
            user_id = int(user_id_raw)
            created_at = datetime.fromisoformat(created_at_raw).replace(tzinfo=timezone.utc)
            deleted = boolean_from_str(deleted_raw)
            user_name = empty_str_as_none(user_name)
            user_slug = empty_str_as_none(user_slug)
            gender = empty_str_as_none(gender)
            birthday = date.fromisoformat(birthday_raw) if birthday_raw else None
            location = empty_str_as_none(location)
            about = empty_str_as_none(about)
            website = empty_str_as_none(website)
            karma_level = int(karma_level_raw)

            # Check if there's an avatar for this user
            # TODO

            # Build import request
            if deleted:
                user_type = ImportDeletedUser()
            else:
                assert user_name is not None
                assert user_slug is not None
                user_type = ImportExistingUser(
                    name=user_name,
                    slug=user_slug,
                )

            request = ImportUserData(
                user_id=user_id,
                created_at=created_at,
                fetched_at=_, # TODO
                wikidot_user_type=user_type,
                real_name=real_name,
                gender=gender,
                birthday=birthday,
                location=location,
                biography=biography,
                website=website,
                karma=karma_level,
                is_pro=account_type == "Pro",
                importing_user_id=_, # TODO
                ip_address=_, # TODO
            )
