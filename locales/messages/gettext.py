#
# gettext.py - Wikijump Locale Builder
#

"""
Utilities for generating and compiling gettext files.
"""

import json
import re
import subprocess
from typing import Iterable, Optional

from .messages import Messages

# For extracting individual comment lines:
# - Ignore any lines not starting with '##'
# - Strip whitespace
# - Strip comment header
DOC_COMMENT_LINE_REGEX = re.compile(r"\s*##\s*(.+)\s*")

# Utilities


def escape_string(string: str) -> str:
    # Unfortunately, codecs.getencoder("unicode_escape") doesn't escape quotes,
    # which we need for them to not break .po file strings.

    return json.dumps(string)


def extract_comment(comment: Optional[str]) -> Iterable[str]:
    if comment is None:
        return ()

    # Process each line, discarding invalid doc comments, adjusting for PO
    lines = []
    for line in comment.splitlines():
        match = DOC_COMMENT_LINE_REGEX.match(line)
        if match is None:
            continue

        lines.append(f"#. {match[1]}")

    return lines


# Main functions


def generate_po(messages: Messages) -> str:
    lines = []

    for path in messages.schema:
        # PO fields:
        # (from https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html#PO-Files)
        #
        #   - #  translator-comments
        #   - #. extracted-comments
        #   - #: reference…
        #   - #, flag…

        message, comment = messages[path]

        lines.extend(extract_comment(comment))  # Add extracted comments
        lines.append("#, python-format")  # Because it uses {..} formatting
        lines.append(f"msgid {escape_string(path)}")
        lines.append(f"msgstr {escape_string(message)}")
        lines.append("")

    return "\n".join(lines)


def build_mo(input_path: str, output_path: str):
    command = [
        "msgfmt",
        "--strict",
        "--output-file",
        output_path,
        input_path,
    ]

    subprocess.check_call(command)
