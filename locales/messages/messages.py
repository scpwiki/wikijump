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

    def __init__(
        name: str,
        language: str,
        country: Optional[str],
        tree: MessagesTree,
    ):
        self.name = name
        self.language = language
        self.country = country
        self.data = flatten(tree)

    def __get__(self, path: str) -> str:
        return self.data[path]


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
