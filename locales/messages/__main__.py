#
# __main__.py - Wikijump Locale Builder
#

"""
Executable file, permitting command-line building of messages files.
"""

import os
import sys

from .catalog import get_template_messages
from .gettext import build_mo, generate_po
from .path_loader import OUTPUT_DIRECTORY, load
from .schema import APPLICATION_NAME, MAIN_MESSAGE_SCHEMA_NAME, validate_all

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

        for mismatch in invalid:
            print(f"- {mismatch.name}")

            if mismatch.unused_fields:
                print("    Unused Fields:")
                for field in mismatch.unused_fields:
                    print(f"     - {field}")

            if mismatch.missing_fields:
                print("    Missing Fields:")
                for field in mismatch.missing_fields:
                    print(f"     - {field}")

        sys.exit(1)

    # Create output directory, if it doesn't exist
    output_directory = os.path.join(directory, OUTPUT_DIRECTORY)
    if not os.path.isdir(output_directory):
        os.makedirs(output_directory)

    # Helper functions
    def get_path(filename):
        return os.path.join(output_directory, filename)

    def write_file(filename, contents):
        print(f"+ {filename}")
        path = get_path(filename)
        with open(path, "w") as file:
            file.write(contents)

    print(f"Generating {len(messages_map) + 1} localization files...")

    # Generate .pot file (template)
    schema = messages_map[MAIN_MESSAGE_SCHEMA_NAME].schema
    template_pot = generate_po(get_template_messages(schema))
    write_file("template.pot", template_pot)

    # Generate .po files
    for name, messages in messages_map.items():
        output_po = generate_po(messages)
        write_file(f"{name}.po", output_po)

    print()
    print(f"Building {len(messages_map)} localization files...")

    # Build .mo files
    for name in messages_map:
        print(f"+ {name}.mo")
        input_path = get_path(f"{name}.po")
        output_path = get_path(f"{name}.mo")
        build_mo(input_path, output_path)

    # Finished
    print()
    print("Done!")
