#!/usr/bin/env python3

import argparse

if __name__ == "__main__":
    argparser = argparse.ArgumentParser(description="WikiComma import utility")
    argparser.add_argument(
        "-o",
        "--output",
        dest="output",
        default="wikicomma_ingest.sql",
        help="The path to write the output SQL file to.",
    )
    argparser.add_argument(
        "database_url",
        help=(
            "A PostgreSQL connection string which has a DEEPWELL database. "
            "The database is not written to."
        ),
    )
    argparser.add_argument(
        "wikicomma_directory",
        help=(
            "The directory containing WikiComma data. "
            "Each top-level directory contains one wiki to be imported."
        ),
    )
    args = argparser.parse_args()

    print(args)
