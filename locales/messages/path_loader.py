#
# loader.py - Wikijump Locale Builder
#

import os
import re
from collections import namedtuple
from graphlib import TopologicalSorter

import yaml

from .messages import Messages

MESSAGE_FILENAME_REGEX = re.compile("(([a-z]+)(?:_([A-Z]+))?)\.ya?ml")

IGNORE_PATHS = [
    ".gitignore",
    "README.md",
    "messages",
]

MessagesStub = namedtuple("MessageStub", ("language", "country", "path"))


def load(directory) -> dict[str, Messages]:
    # Preload all messages to get dependency order
    stubs = {}
    dependencies = TopologicalSorter()

    for filename in os.listdir(directory):
        if filename in IGNORE_PATHS:
            continue

        match = MESSAGE_FILENAME_REGEX.match(filename)
        if match is None:
            print(f"Skipping non-message file '{filename}'.")
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
    messages_map = {}

    for name in dependencies.static_order():
        print(f"+ Loading {name}")
        stub = stubs[name]

        with open(stub.path) as file:
            data = yaml.safe_load(file)

        # If there's a parent, then get that data
        if stub.country is not None:
            parent = messages_map[stub.language]
            parent_data = parent.message_data
            data = {**parent_data, **data}

        # Build messages object
        messages_map[name] = Messages(name, language, country, data)

    return messages_map
