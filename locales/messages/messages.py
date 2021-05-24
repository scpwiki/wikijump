#
# messages.py - Wikijump Locale Builder
#

"""
Represents a messages object, as loaded from configuration.

Includes any data loaded from parent object(s).

Also contains utilities to transform nested messages data
into a flat, path-based mapping.
"""

from dataclasses import dataclass
from typing import Optional, Union

MessagesData = dict[str, str]
MessagesTree = dict[str, Union[str, "MessagesTree"]]


@dataclass
class Messages:
    name: str
    language: str
    country: Optional[str]
    data: MessagesData

    def get(self, path: str) -> str:
        """
        Retrieve the message with the given path.
        """

        data = self.message_data
        parts = path.split(".")

        for i, part in enumerate(parts):
            data = data.get(part)
            if data is None:
                raise KeyError(path)

            if isinstance(data, str):
                # Check that we're at the end of the path
                if i < len(parts) - 1:
                    raise KeyError(path)

                return data

        # Went through the entire path without finding the string
        raise KeyError(path)


class MessagesSchema(set[str]):
    def __init__(self, data: MessagesData):
        keys = set(data.keys())
        super().__init__(keys)


def flatten(tree: MessagesTree) -> MessagesData:
    flattened = {}

    def sub_flatten(prefix: str, tree: MessagesTree):
        for name, child in tree.items():
            path = f"{prefix}.{name}"

            if isinstance(child, str):
                # Leaf object
                flattened[path] = child
            else:
                # Sub-tree
                sub_flatten(path, child)

    return flattened
