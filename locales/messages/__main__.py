#
# __main__.py - Wikijump Locale Builder
#

from .path_loader import load
from .schema import validate_all

import sys

"""
Executable file, permitting command-line building of messages files.
"""


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <locale-directory>")
        sys.exit(1)

    directory = sys.argv[1]
    messages_map = load(directory)

    invalid = validate_all(messages_map)
    if invalid:
        print("The following messages files do not match the schema:")

        for name in invalid:
            print(f"- {name}")

        sys.exit(1)
