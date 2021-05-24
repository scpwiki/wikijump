#
# gettext.py - Wikijump Locale Builder
#

"""
Utilities for generating and compiling gettext files.
"""

import os
import subprocess

from .messages import Messages


def generate_po(messages: Messages) -> str:
    lines = []

    for path, message in messages.data:
        # TODO add comments
        # - #  translator-comments
        # - #. extracted-comments
        # - #: reference… (won't use)
        # - #, flag… (won't use)
        lines.append(f"msgid {path!r}")
        lines.append(f"msgstr {message!r}")
        lines.append("")

    return "\n".join(lines)


def build_po(input
