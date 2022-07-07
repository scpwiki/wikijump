#!/usr/bin/env python3

import argparse
import logging
import sys
from importer import run_wikicomma_import

LOG_FORMAT = "[%(levelname)s] %(asctime)s %(name)s: %(message)s"
LOG_DATE_FORMAT = "[%Y/%m/%d %H:%M:%S]"

if __name__ == "__main__":
    argparser = argparse.ArgumentParser(description="WikiComma importer")
    argparser.add_argument(
        "-q",
        "--quiet",
        "--no-stdout",
        dest="stdout",
        action="store_false",
        help="Don't output to standard out.",
    )
    argparser.add_argument(
        "-D",
        "--debug",
        dest="debug",
        action="store_true",
        help="Set logging level to debug.",
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
        "--sql",
        "--output-sql",
        dest="sql_path",
        required=True,
        help="The location to output the SQL dump to",
    )
    argparser.add_argument(
        "-s",
        "--shell",
        "--output-shell",
        dest="sh_path",
        required=True,
        help="The location to output the shell dump to",
    )
    argparser.add_argument(
        "-b",
        "--s3",
        "--s3-bucket",
        dest="s3_bucket",
        required=True,
        help="The name of the S3 bucket to use (read-only)",
    )
    argparser.add_argument(
        "-u",
        "--postgres-url",
        dest="postgres_url",
        required=True,
        help="The DEEPWELL database to connect to (read-only)",
    )
    args = argparser.parse_args()

    log_fmtr = logging.Formatter(LOG_FORMAT, datefmt=LOG_DATE_FORMAT)
    log_stdout = logging.StreamHandler(sys.stdout)
    log_stdout.setFormatter(log_fmtr)
    log_level = logging.DEBUG if args.debug else logging.INFO

    logger = logging.getLogger("importer")
    logger.setLevel(level=log_level)
    logger.addHandler(log_stdout)

    run_wikicomma_import(
        wikicomma_directory=args.wikicomma_directory,
        sql_path=args.sql_path,
        sh_path=args.sh_path,
        s3_bucket=args.s3_bucket,
        postgres_url=args.postgres_url,
    )
