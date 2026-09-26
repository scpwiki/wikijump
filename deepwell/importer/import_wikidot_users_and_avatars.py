#!/usr/bin/env python

"""
Import script to read in a wikidot_users.csv file and associated avatars.
"""

import argparse
import lzma

from deepwell_common import Deepwell


class Importer:
    def __init__(self, deepwell: Deepwell) -> None:
        self.deepwell = deepwell


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
        "users_csv_file",
        help="wikidot_users.csv input file",
    )
    args = argparser.parse_args()

    deepwell = Deepwell(args.host, args.port)
    importer = Importer(deepwell)
