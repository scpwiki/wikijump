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
from functools import cached_property
from typing import Optional, Union

MessagesData = dict[str, str]
MessagesTree = dict[str, Union[str, "MessagesTree"]]


@dataclass
class Messages:
    name: str
    language: str
    country: Optional[str]
    data: MessagesData

    @cached_property
    def schema(self) -> MessagesSchema:
        keys = frozenset(self.data.keys())
        return MessagesSchema(keys)

    def __get__(self, path: str) -> str:
        return self.data[path]


class MessagesSchema(frozenset[str]):
    def validate(other: MessagesSchema) -> bool:
        """
        Determines if this schema is valid.

        That is, it checks if the other schema is
        equal to or a subset of this one, which is considered the authority.
        """

        return self >= other


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
