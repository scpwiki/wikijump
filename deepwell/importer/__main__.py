#!/usr/bin/env python3


if __name__ == "__main__":
    # Parse arguments
    argparser = argparse.ArgumentParser(description="WikiComma import utility")
    argparser.add_argument(
        "--start-site-id",
        dest="start_site_id",
        default=3,
        help="What ID value to start enumerating new sites from",
    )
    argparser.add_argument(
        "--start-category-id",
        dest="start_category_id",
        default=3,
        help="What ID value to start enumerating new page categories from",
    )
    argparser.add_argument(
        "-o",
        "--output",
        dest="output_file",
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

    # Run importer
    WikicommaImporter(args).run()
