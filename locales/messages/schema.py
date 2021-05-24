#
# schema.py - Wikijump Locale Builder
#

"""
Represents the schema that a messages file has.

Can be used to verify that another data file matches,
and to generate templates.
"""

from typing import List

# The messages file to use as the "schema".
# That is, this file is complete and can be used to build a schema.
MESSAGE_SCHEMA_NAME = "en"


class MessagesSchema(frozenset[str]):
    """
    Represents a messages schema.

    It contains an immutable set of all paths within the data object,
    after it has been flattened.
    """

    def validate(other: MessagesSchema) -> bool:
        """
        Determines if this schema is valid.

        That is, it checks if the other schema is
        equal to or a subset of this one, which is considered the authority.
        """

        return self >= other


def validate_all(messages_map: dict[str, Messages]) -> List[str]:
    invalid = []

    # Get main schema
    main_schema = messages_map[MESSAGE_SCHEMA_NAME].get_schema()

    # Check all messages in the mapping for compliance
    for name, messages in messages_map.items():
        schema = messages.get_schema()

        if not main_schema.validate(schema):
            invalid.append(name)

    # Return sorted list of invalid messages objects
    invalid.sort()
    return invalid
