#
# __main__.py - Wikijump Locale Builder
#

"""
Executable file, permitting command-line building of messages files.
"""

from .gettext import build
from .messages import get_template_messages
from .path_loader import load
from .schema import MAIN_MESSAGE_SCHEMA_NAME, validate_all

import sys

# The directory generated artifacts should go in.
OUTPUT_DIRECTORY_NAME = "out"

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <locale-directory>")
        sys.exit(1)

    # Get messages
    print("Loading localizations...")
    directory = sys.argv[1]
    messages_map = load(directory)
    print()

    # Validate schemas
    invalid = validate_all(messages_map)
    if invalid:
        print("The following localizations do not match the schema:")

        for name in invalid:
            print(f"- {name}")

        sys.exit(1)

    # Create output directory, if it doesn't exist
    output_directory = os.path.join(directory, OUTPUT_DIRECTORY_NAME)
    if not os.path.isdir(output_directory):
        os.makedirs(output_directory)

    # Helper for writing output files
    def write_file(filename, contents):
        path = os.path.join(output_directory, filename)

        with open(path, "w") as file:
            file.write(contents)

    # Generate .po and .pot files
    print("Generating template localization file...")
    schema = messages_map[MAIN_MESSAGE_SCHEMA_NAME].schema
    template_pot = generate_po(get_template_messages(schema))
    write_file("template.pot", template_pot)

    print(f"Generate {len(messages_map)} localization files...")
    # TODO
