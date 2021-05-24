#
# __main__.py - Wikijump Locale Builder
#

from .path_loader import load

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
