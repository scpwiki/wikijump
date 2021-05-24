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
from typing import Iterable, Optional, Tuple, Union

from ruamel.yaml.compat import ordereddict

from .schema import MessagesSchema

CommentData = dict[str, Optional[str]]
MessagesData = dict[str, str]
MessagesTree = ordereddict[str, Union[str, "MessagesTree"]]


@dataclass
class Messages:
    name: str
    language: str
    country: Optional[str]
    message_data: MessagesData
    comment_data: CommentData

    @property
    def schema(self) -> MessagesSchema:
        return self.message_data.keys()

    def __getitem__(self, path: str) -> Tuple[str, Optional[str]]:
        message = self.message_data[path]
        comment = self.comment_data[path]
        return message, comment


def flatten(tree: MessagesTree) -> Tuple[MessagesData, CommentData]:
    """
    Flattens the given messages tree into a mapping of path to value.
    """

    flattened = {}
    comments = {}

    def get_comment(tree: ordereddict, key: str):
        comments = tree.ca.items
        if key in comments:
            _, _, comment, _ = comments[key]
            return comment.value
        else:
            return None

    def sub_flatten(prefix: Optional[str], tree: MessagesTree):
        for name, child in tree.items():
            if prefix is None:
                path = name
            else:
                path = f"{prefix}.{name}"

            if isinstance(child, str):
                # Leaf object
                flattened[path] = child.strip()
                comments[path] = get_comment(tree, name)
            else:
                # Sub-tree
                sub_flatten(path, child)

    sub_flatten(None, tree)
    return flattened, comments


def get_template_messages(schema: Iterable[str]) -> Messages:
    """
    Create a dummy Messages object for the given schema.
    """

    message_data = {path: "" for path in schema}
    comment_data = {path: None for path in schema}
    return Messages("template", "template", None, message_data, comment_data)
