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
from typing import Iterable, Optional, Union

from .schema import MessagesSchema

MessagesData = dict[str, str]
MessagesTree = dict[str, Union[str, "MessagesTree"]]


@dataclass
class Messages:
    name: str
    language: str
    country: Optional[str]
    data: MessagesData

    @property
    def schema(self) -> MessagesSchema:
        return self.data.keys()

    def __get__(self, path: str) -> str:
        return self.data[path]


def flatten(tree: MessagesTree) -> MessagesData:
    """
    Flattens the given messages tree into a mapping of path to value.
    """

    flattened = {}

    def sub_flatten(prefix: Optional[str], tree: MessagesTree):
        for name, child in tree.items():
            if prefix is None:
                path = name
            else:
                path = f"{prefix}.{name}"

            if isinstance(child, str):
                # Leaf object
                flattened[path] = child
            else:
                # Sub-tree
                sub_flatten(path, child)

    sub_flatten(None, tree)
    return flattened


def get_template_messages(schema: Iterable[str]) -> Messages:
    """
    Create a dummy Messages object for the given schema.
    """

    data = {path: "" for path in schema}
    return Messages("template", "template", None, data)
