#
# __main__.py - Wikijump Locale Builder
#

from .messages import MESSAGE_FILENAME_REGEX, Messages

import os
import sys

IGNORE_PATHS = ["build"]


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <locale-directory>")
        sys.exit(1)

    directory = sys.argv[1]
    for filename in os.listdir(directory):
        if filename in IGNORE_PATHS:
            continue

        match = MESSAGE_FILENAME_REGEX.match(filename)
        if match is None:
            print(f"Skipping non-message file '{filename}'.")
            continue

        language = match[1]
        country = match[2]
