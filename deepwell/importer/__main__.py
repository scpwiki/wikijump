#!/usr/bin/env python3

import argparse
import logging
import os
import sys

from .importer import Importer

LOG_FORMAT = "[%(levelname)s] [%(asctime)s] %(message)s"
LOG_DATE_FORMAT = "%Y/%m/%d %H:%M:%S"

if __name__ == "__main__":
    argparser = argparse.ArgumentParser(description="WikiComma importer")
    argparser.add_argument(
        "-q",
        "--quiet",
        "--no-stdout",
        dest="stdout",
        action="store_false",
        help="Don't output to standard out",
    )
    argparser.add_argument(
        "-d",
        "--directory",
        "--wikicomma-directory",
        dest="wikicomma_directory",
        required=True,
        help="The directory where WikiComma data resides",
    )
    argparser.add_argument(
        "-o",
        "--sqlite",
        "--output-sqlite",
        dest="sqlite_path",
        required=True,
        help="The location to output the SQLite database to",
    )
    argparser.add_argument(
        "-D",
        "--delete-sqlite",
        action="store_true",
        help="Delete the output SQLite before starting operations",
    )
    argparser.add_argument(
        "-b",
        "--bucket",
        "--s3-bucket",
        dest="s3_bucket",
        required=True,
        help="The S3 bucket to store uploaded files in",
    )
    argparser.add_argument(
        "-P",
        "--profile",
        "--aws-profile",
        dest="aws_profile",
        required=True,
        help="The AWS profile containing the secrets",
    )
    args = argparser.parse_args()

    log_fmtr = logging.Formatter(LOG_FORMAT, datefmt=LOG_DATE_FORMAT)
    log_stdout = logging.StreamHandler(sys.stdout)
    log_stdout.setFormatter(log_fmtr)

    logger = logging.getLogger(__package__)
    logger.setLevel(level=logging.DEBUG)
    logger.addHandler(log_stdout)

    importer = Importer(
        wikicomma_directory=args.wikicomma_directory,
        sqlite_path=args.sqlite_path,
        delete_sqlite=args.delete_sqlite,
        s3_bucket=args.s3_bucket,
        aws_profile=args.aws_profile,
    )
    importer.run()
