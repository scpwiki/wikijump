from collections import namedtuple

Page = nametuple("Page", ("slug", "wikidot_id"))
Site = namedtuple("Site", ("slug", "wikijump_id"))
User = namedtuple("User", ("slug", "wikidot_id"))
