#
# __init__.py - Wikijump Locale Builder
#

"""
Parent file for the messages module.
"""

from . import schema
from .path_loader import load
from .schema import MessagesSchema
from .messages import Messages, MessagesData, MessagesTree
