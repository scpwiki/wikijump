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
from glob import iglob
from typing import NotRequired, TypedDict

from deepwell_common import Deepwell


class WikicommaUserRecord(TypedDict):
    full_name: str
    username: str
    wikidot_user_since: int
    account_type: str
    activity: int
    fetched_at: int
    user_id: int


def read_wikicomma_users(json_path: str) -> Iterator[WikicommaUserRecord]:
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
    # TODO
    ...


if __name__ == "__main__":
    argparser = argparse.ArgumentParser("import_missing_wikidot_users")
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

    deepwell = Deepwell(args.host, args.port)

    json_file_glob = os.path.join(args.wikicomma_directory, "*.json")
    for json_path in iglob(json_file_glob):
        for user in read_wikicomma_users(json_path):
            import_user_if_missing(
                deepwell,
                user,
                importer_user_id=args.importer_user_id,
                importer_ip_address=args.importer_ip_address,
            )
