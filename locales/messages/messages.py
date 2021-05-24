#
# messages.py - Wikijump Locale Builder
#

"""
Represents a messages object, as loaded from configuration.

Includes any data loaded from parent object(s).
"""

from dataclasses import dataclass
from typing import Optional, Union

MessagesData = dict[str, Union[str, "MessagesData"]]


@dataclass
class Messages:
    name: str
    language: str
    country: Optional[str]
    message_data: MessagesData

    def get(self, path) -> str:
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
