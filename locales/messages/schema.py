#
# schema.py - Wikijump Locale Builder
#

"""
Represents the schema that a messages file has.

Can be used to verify that another data file matches,
and to generate templates.
"""

from typing import Iterable, List, NewType

# The messages file to use as the "schema".
# That is, this file is complete and can be used to build a schema.
MAIN_MESSAGE_SCHEMA_NAME = "en"

# Represents a messages schema.
#
# It contains an immutable set of all paths within the data object,
# after it has been flattened.
MessagesSchema = NewType("MessagesSchema", Iterable[str])


def validate_all(
    messages_map: dict[str, "Messages"],
    main_schema_name=MAIN_MESSAGE_SCHEMA_NAME,
) -> List[str]:
    """
    Validate all messages within the mapping against the main schema.
    """

    # Get main schema
    main_schema = messages_map[main_schema_name].schema

    # Check all messages in the mapping for compliance
    invalid = []
    for name, messages in messages_map.items():
        if main_schema != messages.schema:
            invalid.append(name)

    # Return sorted list of invalid messages objects
    invalid.sort()
    return invalid
