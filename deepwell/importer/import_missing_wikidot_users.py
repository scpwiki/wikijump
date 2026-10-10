#!/usr/bin/env python

"""
Import script for missing user data gathered by wikicomma.

As in, this script inserts users which are present in wikicomma's dataset
but was not added by import_wikidot_users_and_avatars.py
"""

import argparse
import json
import os
from collections.abc import Iterator
from datetime import datetime, timezone
from glob import iglob
from typing import NotRequired, TypedDict

from deepwell_common import Deepwell, ImportExistingUser, ImportUserData
from log_common import setup_logging

# can't use declarative syntax because 'from' is a keyword
WikicommaUserRecord = TypedDict(
    "WikicommaUserRecord",
    {
        "full_name": str,
        "username": str,
        "real_name": NotRequired[str],
        "gender": NotRequired[bool | str],
        "birthday": NotRequired[int],  # JS timestamp, so millis
        "from": NotRequired[str],
        "website": NotRequired[str],
        "wikidot_user_since": int,  # regular, seconds
        "account_type": str,
        "activity": int,
        "fetched_at": int,  # also millis
        "user_id": int,
    },
)


def convert_gender(value: str | bool | None) -> str | None:
    # in some cases gender is stored as bool
    match value:
        case "male" | True:
            return "male"
        case "female" | False:
            return "female"
        case None:
            return None
        case _:
            raise ValueError(value)


def read_wikicomma_users(json_path: str) -> Iterator[WikicommaUserRecord]:
    if os.path.basename(json_path) == "pending.json":
        # not a user list
        return

    with open(json_path) as file:
        users_data = json.load(file)
        yield from users_data.values()


def import_user_if_missing(
    deepwell: Deepwell,
    user: WikicommaUserRecord,
    *,
    importer_user_id: int,
    importer_ip_address: str,
) -> None:
    if deepwell.user_exists(user["user_id"]):
        # nothing to do
        logger.debug("Skipping user ID %d, already in dataset", user["user_id"])
        return

    user_type = ImportExistingUser(
        name=user["full_name"],
        slug=user["username"],
    )
    created_at = datetime.fromtimestamp(user["wikidot_user_since"], tz=timezone.utc)
    fetched_at = datetime.fromtimestamp(user["fetched_at"] // 1000, tz=timezone.utc)
    birthday = (
        datetime.fromtimestamp(user["birthday"] // 1000, tz=timezone.utc)
        if "birthday" in user
        else None
    )

    request = ImportUserData(
        user_id=user["user_id"],
        created_at=created_at,
        fetched_at=fetched_at,
        wikidot_user_type=user_type,
        avatar_uploaded_blob_id=None,
        real_name=user.get("real_name"),
        gender=convert_gender(user.get("gender")),
        birthday=birthday,
        location=user.get("from"),
        biography="",  # not available
        website=user.get("website"),
        karma=user["activity"],
        is_pro=False,  # not available
        upsert=False,
        importing_user_id=importer_user_id,
        ip_address=importer_ip_address,
    )
    deepwell.import_user(request)


if __name__ == "__main__":
    argparser = argparse.ArgumentParser("import_missing_wikidot_users")
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
        "wikicomma_directory",
        help="The _users directory produced by wikicomma",
    )
    args = argparser.parse_args()

    logger = setup_logging(args.debug)
    logger.info("Running import_missing_wikidot_users with configuration:")
    logger.info("* DEEPWELL server:        %s %d", args.host, args.port)
    logger.info("* Wikicomma directory:    %s", args.wikicomma_directory)

    deepwell = Deepwell(args.host, args.port)

    json_file_glob = os.path.join(args.wikicomma_directory, "*.json")
    for json_path in iglob(json_file_glob):
        logger.info("Reading JSON file %s", json_path)
        for user in read_wikicomma_users(json_path):
            import_user_if_missing(
                deepwell,
                user,
                importer_user_id=args.importer_user_id,
                importer_ip_address=args.importer_ip_address,
            )
