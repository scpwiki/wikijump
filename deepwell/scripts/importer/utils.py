from psycopg2.extensions import register_adapter, AsIs


def get_page_category(page_slug):
    parts = page_slug.split(":")
    if len(parts) == 1:
        return "_default"

    return parts[0]


class SqlRaw:
    __slots__ = ("value",)

    def __init__(self, value: str):
        self.value = value

    def adapt(self):
        return AsIs(self.value)


def wikidot_id_or_auto(item):
    if item.wikidot_id is None:
        return SqlRaw("DEFAULT")
    else:
        return item.wikidot_id


register_adapter(SqlRaw, SqlRaw.adapt)
