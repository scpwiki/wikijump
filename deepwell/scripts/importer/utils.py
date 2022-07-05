from psycopg2 import sql


def get_page_category(page_slug):
    parts = page_slug.split(":")
    if len(parts) == 1:
        return "_default"

    return parts[0]


def wikidot_id_or_auto(item):
    if item.wikidot_id is None:
        return sql.DEFAULT
    else:
        return item.wikidot_id
