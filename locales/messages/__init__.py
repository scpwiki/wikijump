#
# __init__.py - Wikijump Locale Builder
#

"""
Parent file for the messages module.
"""

from . import catalog, gettext, schema
from .path_loader import load
from .schema import MessagesSchema
