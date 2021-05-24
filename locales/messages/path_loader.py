#
# loader.py - Wikijump Locale Builder
#

import os
import re
from collections import namedtuple
from graphlib import TopologicalSorter

from ruamel.yaml import YAML

from .messages import Messages, flatten

"""
Load message data from files, including any language inheritance.
"""

MESSAGE_FILENAME_REGEX = re.compile("(([a-z]+)(?:_([A-Z]+))?)\.ya?ml")

OUTPUT_DIRECTORY = "out"

# Files in this directory which aren't messages files.
# Anything else gets a warning printed.
IGNORE_PATHS = [
    ".gitignore",
    "README.md",
    "messages",
    OUTPUT_DIRECTORY,
]

# Represents a messages file that has not yet been read.
MessagesStub = namedtuple("MessageStub", ("language", "country", "path"))


def load(directory: str, log=True) -> dict[str, Messages]:
    # Preload all messages to get dependency order
    stubs = {}
    dependencies = TopologicalSorter()

    for filename in os.listdir(directory):
        if filename in IGNORE_PATHS:
            continue

        match = MESSAGE_FILENAME_REGEX.match(filename)
        if match is None:
            print(f"* Skipping non-message file '{filename}'.")
            continue

        # Build messages stub data
        name = match[1]
        language = match[2]
        country = match[3]
        path = os.path.join(directory, filename)
        stubs[name] = MessagesStub(language, country, path)

        if country is None:
            # No dependencies
            dependencies.add(name)
        else:
            # Requires base language
            dependencies.add(name, language)

    # Load all messages in order to apply inheritance
    yaml = YAML()
    messages_map = {}

    for name in dependencies.static_order():
        if log:
            print(f"+ {name}")

        stub = stubs[name]
        with open(stub.path) as file:
            tree = yaml.load(file)

        # Get path -> message mapping
        message_data, comment_data = flatten(tree)

        # If there's a parent, then get that data
        if stub.country is not None:
            parent = messages_map[stub.language]
            message_data = {**parent.message_data, **message_data}
            comment_data = {**parent.comment_data, **comment_data}

        # Build messages object
        messages_map[name] = Messages(
            name, language, country, message_data, comment_data,
        )

    return messages_map
