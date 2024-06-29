#!/usr/bin/env python3

import argparse
import logging
import os
import sys

from .importer import Importer
from .wikicomma_config import parse_config

LOG_FORMAT = "[%(levelname)s] %(asctime)s %(name)s: %(message)s"
LOG_DATE_FORMAT = "%Y/%m/%d %H:%M:%S"
LOG_FILENAME = "import.log"
LOG_FILE_MODE = "a"

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
        "--log",
        dest="log_file",
        default=LOG_FILENAME,
        help="The log file to write to",
    )
    argparser.add_argument(
        "-c",
        "--config",
        dest="wikicomma_config",
        required=True,
        help="The configuration JSON that Wikicomma uses",
    )
    argparser.add_argument(
        "-d",
        "--directory",
        "--wikicomma-directory",
        dest="wikicomma_directory",
        required=True,
        help="The directory where Wikicomma data resides",
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

    log_file = logging.FileHandler(
        filename=LOG_FILENAME,
        encoding="utf-8",
        mode=LOG_FILE_MODE,
    )
    log_file.setFormatter(log_fmtr)

    logger = logging.getLogger(__package__)
    logger.setLevel(level=logging.DEBUG)
    logger.addHandler(log_file)

    if args.stdout:
        log_stdout = logging.StreamHandler(sys.stdout)
        log_stdout.setFormatter(log_fmtr)
        logger.addHandler(log_stdout)

    wikicomma_config = parse_config(args.wikicomma_config)

    importer = Importer(
        wikicomma_config=wikicomma_config,
        wikicomma_directory=args.wikicomma_directory,
        sqlite_path=args.sqlite_path,
        delete_sqlite=args.delete_sqlite,
        s3_bucket=args.s3_bucket,
        aws_profile=args.aws_profile,
    )
    importer.run()
