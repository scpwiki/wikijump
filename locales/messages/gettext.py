#
# gettext.py - Wikijump Locale Builder
#

"""
Utilities for generating and compiling gettext files.
"""

import os
import subprocess
from codecs import getdecoder

from .messages import Messages

unicode_escape = getdecoder("unicode_escape")


def generate_po(messages: Messages) -> str:
    lines = []

    for path, message in messages.data.items():
        # TODO add comments
        # - #  translator-comments
        # - #. extracted-comments
        # - #: reference… (won't use)
        # - #, flag…
        lines.append("#, python-format")  # Because it uses {..} formatting
        lines.append(f'msgid "{escape_string(path)}"')
        lines.append(f'msgstr "{escape_string(message)}"')
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


def escape_string(string: str) -> str:
    return unicode_escape(string)[0]
