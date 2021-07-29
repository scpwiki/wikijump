#!/usr/bin/python3

import json
import random
import time
from collections import defaultdict
from contextlib import contextmanager
from dataclasses import dataclass
from uuid import uuid4

import psycopg2

TAGS = open('tags.txt').read().splitlines()

PAGES = 1000
OPERATIONS = 10000

# Utilities

@dataclass
class Page:
    slug: str
    tags_a: set[str]
    tags_b: set[str]
    page_id_a: int
    page_id_b: int

@contextmanager
def timer(name):
    start = time.monotonic_ns()
    yield
    stop = time.monotonic_ns()
    elapsed_ns = stop - start
    elapsed_ms = elapsed_ns / 1e6
    elapsed_ms_each = elapsed_ms / OPERATIONS
    print(f"+ Task '{name}' in {elapsed_ms:.4f} ms ({elapsed_ms_each:.4f} ms per iteration)")

@contextmanager
def transaction(cur):
    cur.execute("BEGIN")
    yield
    cur.execute("COMMIT")

def random_tags(*, min_len=1, max_len=20):
    return {
        random.choice(TAGS)
        for _ in range(random.randint(min_len, max_len))
    }

def select_tags(tags, *, min_len=1, max_len=20):
    if not tags:
        return set()

    selected_tags = set()
    static_tags = tuple(tags)

    return {
        random.choice(static_tags)
        for _ in range(random.randint(min_len, max_len))
    }

def generate_slugs_and_tags():
    slugs = {}

    for _ in range(PAGES):
        slug = str(uuid4())
        tags = random_tags()

        slugs[slug] = tags

    return slugs

def print_plan(cur):
    for line, in cur.fetchall():
        print('  ', line)

    print()

# Setup

def database_connect(path = 'database.json'):
    with open(path) as file:
        config = json.load(file)

    return psycopg2.connect(
        host=config['host'],
        port=config['port'],
        database=config['database'],
        user=config['user'],
        password=config['password'],
    )

def database_schema_create(cur):
    # Option A
    cur.execute("""
        CREATE TABLE IF NOT EXISTS a__pages (
            page_id BIGSERIAL PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL
        )
    """)

    cur.execute("""
        CREATE TABLE IF NOT EXISTS a__page_tags (
            tag TEXT,
            page_id BIGINT REFERENCES a__pages (page_id),
            PRIMARY KEY (tag, page_id)
        )
    """)

    cur.execute(
        "CREATE INDEX IF NOT EXISTS a__page_tags_tag_idx ON a__page_tags (tag)",
    )

    # Option B
    cur.execute("""
        CREATE TABLE IF NOT EXISTS b__pages (
            page_id BIGSERIAL PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL,
            tags TEXT[] NOT NULL
        )
    """)

def database_populate(cur):
    cur.execute("TRUNCATE a__page_tags")
    cur.execute("TRUNCATE a__pages CASCADE")
    cur.execute("TRUNCATE b__pages")

    slugs = generate_slugs_and_tags()
    pages = {}

    with timer("option A populate"):
        for slug, tags in slugs.items():
            cur.execute(
                "INSERT INTO a__pages (slug) VALUES (%(slug)s) RETURNING page_id",
                {'slug': slug},
            )
            page_id_a, = cur.fetchone()

            for tag in tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                    {'tag': tag, 'page_id': page_id_a},
                )

            pages[slug] = Page(slug, tags, tags, page_id_a, None)

    with timer("option B populate"):
        for slug, tags in slugs.items():
            cur.execute(
                "INSERT INTO b__pages (slug, tags) VALUES (%(slug)s, %(tags)s) RETURNING page_id",
                {'slug': slug, 'tags': list(tags)},
            )
            page_id_b, = cur.fetchone()
            pages[slug].page_id_b = page_id_b

    return pages

def plan_database_populate(cur):
    print("** Query plan for Option A populate")
    cur.execute("EXPLAIN INSERT INTO a__pages (slug) VALUES ('plan') RETURNING page_id")
    print_plan(cur)

    cur.execute("EXPLAIN INSERT INTO a__page_tags (tag, page_id) VALUES ('plan', 1)")
    print_plan(cur)

    print("** Query plan for Option B populate")
    cur.execute("EXPLAIN INSERT INTO b__pages (slug, tags) VALUES ('plan', '{}')")
    print_plan(cur)

def refresh_page_data(cur, pages):
    for slug, page in pages.items():
        cur.execute(
            "SELECT tag FROM a__page_tags WHERE page_id = %(page_id)s",
            {'page_id': page.page_id_a},
        )
        page.tags_a = {tag for tag, in cur.fetchall()}

        cur.execute(
            "SELECT tags FROM b__pages WHERE page_id = %(page_id)s",
            {'page_id': page.page_id_b},
        )
        tags, = cur.fetchone()
        page.tags_b = set(tags)

# Operations

def read_tags(cur, pages, slugs):
    with timer("option A read tags (slug)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)

            cur.execute("""
                    SELECT a__page_tags.tag
                    FROM a__page_tags
                    JOIN a__pages
                        ON a__page_tags.page_id = a__pages.page_id
                    WHERE a__pages.slug = %(slug)s
                """,
                {'slug': slug},
            )
            tags = {tag for (tag,) in cur.fetchall()}
            assert pages[slug].tags_a == tags

    with timer("option A read tags (page id)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)

            cur.execute(
                "SELECT tag FROM a__page_tags WHERE page_id = %(page_id)s",
                {'page_id': pages[slug].page_id_a},
            )
            tags = {tag for (tag,) in cur.fetchall()}
            assert pages[slug].tags_a == tags

    with timer("option B read tags"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)

            cur.execute("SELECT tags FROM b__pages WHERE slug = %(slug)s", {'slug': slug})
            tags_raw, = cur.fetchone()
            tags = set(tags_raw)
            assert pages[slug].tags_b == tags

def plan_read_tags(cur):
    print("** Query plan for Option A read tags (slug)")
    cur.execute("""
        EXPLAIN SELECT a__page_tags.tag
        FROM a__page_tags
        JOIN a__pages
            ON a__page_tags.page_id = a__pages.page_id
        WHERE a__pages.slug = 'plan'
    """)
    print_plan(cur)

    print("** Query plan for Option A read tags (page id)")
    cur.execute("EXPLAIN SELECT tag FROM a__page_tags WHERE page_id = 1")
    print_plan(cur)

    print("** Query plan for Option B read tags")
    cur.execute("EXPLAIN SELECT tags FROM b__pages WHERE slug = 'plan'")
    print_plan(cur)

def add_tags(cur, pages, slugs):
    refresh_page_data(cur, pages)

    with timer("option A add tags (slug)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_a
            added_tags = random_tags(max_len=10)
            added_tags.difference_update(tags)

            cur.execute("SELECT page_id FROM a__pages WHERE slug = %(slug)s", {'slug': slug})
            page_id_a, = cur.fetchone()
            assert pages[slug].page_id_a == page_id_a

            for tag in added_tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.update(added_tags)

    with timer("option A add tags (page id)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_a
            added_tags = random_tags(max_len=10)
            added_tags.difference_update(tags)
            page_id_a = pages[slug].page_id_a

            for tag in added_tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.update(added_tags)

    with timer("option B add tags"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            added_tags = random_tags(max_len=10)
            tags = pages[slug].tags_b
            tags.update(added_tags)

            cur.execute(
                "UPDATE b__pages SET tags = %(tags)s WHERE slug = %(slug)s",
                {'tags': list(tags), 'slug': slug},
            )

def plan_add_tags(cur):
    print("** Query plan for Option A add tags")
    cur.execute("EXPLAIN INSERT INTO a__page_tags (tag, page_id) VALUES ('plan', 1)")
    print_plan(cur)

    print("** Query plan for Option B add tags")
    cur.execute("EXPLAIN SELECT tags FROM b__pages WHERE slug = 'plan'")
    print_plan(cur)

def remove_tags(cur, pages, slugs):
    refresh_page_data(cur, pages)

    with timer("option A remove tags (slug)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_a
            removed_tags = select_tags(tags)

            cur.execute("SELECT page_id FROM a__pages WHERE slug = %(slug)s", {'slug': slug})
            page_id_a, = cur.fetchone()
            assert pages[slug].page_id_a == page_id_a

            for tag in removed_tags:
                cur.execute(
                    "DELETE FROM a__page_tags WHERE tag = %(tag)s AND page_id = %(page_id)s",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.difference_update(removed_tags)

    with timer("option A remove tags (page id)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            page_id_a = pages[slug].page_id_a
            tags = pages[slug].tags_a
            removed_tags = select_tags(tags)

            for tag in removed_tags:
                cur.execute(
                    "DELETE FROM a__page_tags WHERE tag = %(tag)s AND page_id = %(page_id)s",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.difference_update(removed_tags)

    with timer("option B remove tags"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            removed_tags = select_tags(tags)
            tags = pages[slug].tags_b

            tags.difference_update(removed_tags)

            cur.execute(
                "UPDATE b__pages SET tags = %(tags)s WHERE slug = %(slug)s",
                {'tags': list(tags), 'slug': slug},
            )

def plan_remove_tags(cur):
    print("** Query plan for Option A remove tags")
    cur.execute("EXPLAIN DELETE FROM a__page_tags WHERE tag = 'plan' AND page_id = 1")
    print_plan(cur)

    print("** Query plan for Option B remove tags")
    cur.execute("EXPLAIN UPDATE b__pages SET tags = '{}' WHERE slug = 'plan'")
    print_plan(cur)

def change_tags(cur, pages, slugs):
    refresh_page_data(cur, pages)

    with timer("option A change tags (slug)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_a
            added_tags = random_tags(max_len=10)
            added_tags.difference_update(tags)
            removed_tags = select_tags(tags)

            cur.execute("SELECT page_id FROM a__pages WHERE slug = %(slug)s", {'slug': slug})
            page_id_a, = cur.fetchone()
            assert pages[slug].page_id_a == page_id_a

            for tag in added_tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                    {'tag': tag, 'page_id': page_id_a},
                )

            for tag in removed_tags:
                cur.execute(
                    "DELETE FROM a__page_tags WHERE tag = %(tag)s AND page_id = %(page_id)s",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.update(added_tags)
            tags.difference_update(removed_tags)

    with timer("option A change tags (page id)"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_a
            added_tags = random_tags(max_len=10)
            added_tags.difference_update(tags)
            removed_tags = select_tags(tags)
            page_id_a = pages[slug].page_id_a

            for tag in added_tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                    {'tag': tag, 'page_id': page_id_a},
                )

            for tag in removed_tags:
                cur.execute(
                    "DELETE FROM a__page_tags WHERE tag = %(tag)s AND page_id = %(page_id)s",
                    {'tag': tag, 'page_id': page_id_a},
                )

            tags.update(added_tags)
            tags.difference_update(removed_tags)

    with timer("option B change tags"):
        for _ in range(OPERATIONS):
            slug = random.choice(slugs)
            tags = pages[slug].tags_b
            added_tags = random_tags(max_len=10)
            added_tags.difference_update(tags)
            removed_tags = select_tags(tags)

            tags.update(added_tags)
            tags.difference_update(removed_tags)

            cur.execute(
                "UPDATE b__pages SET tags = %(tags)s WHERE slug = %(slug)s",
                {'tags': list(tags), 'slug': slug},
            )

def overwrite_tags(cur, pages, slugs):
    with timer("option A overwrite tags (slug)"):
        for _ in range(OPERATIONS):
            with transaction(cur):
                slug = random.choice(slugs)
                new_tags = random_tags()
                pages[slug].tags_a = new_tags

                cur.execute("SELECT page_id FROM a__pages WHERE slug = %(slug)s", {'slug': slug})
                page_id_a, = cur.fetchone()
                assert pages[slug].page_id_a == page_id_a

                cur.execute(
                    "DELETE FROM a__page_tags WHERE page_id = %(page_id)s",
                    {'page_id': page_id_a},
                )

                for tag in new_tags:
                    cur.execute(
                        "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                        {'tag': tag, 'page_id': page_id_a},
                    )

    with timer("option A overwrite tags (page id)"):
        for _ in range(OPERATIONS):
            with transaction(cur):
                slug = random.choice(slugs)
                new_tags = random_tags()
                pages[slug].tags_a = new_tags

                cur.execute(
                    "DELETE FROM a__page_tags WHERE page_id = %(page_id)s",
                    {'page_id': pages[slug].page_id_a},
                )

                for tag in new_tags:
                    cur.execute(
                        "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)s)",
                        {'tag': tag, 'page_id': pages[slug].page_id_a},
                    )

    with timer("option B overwrite tags"):
        for _ in range(OPERATIONS):
            with transaction(cur):
                slug = random.choice(slugs)
                new_tags = random_tags()
                pages[slug].tags_b = new_tags

                cur.execute(
                    "UPDATE b__pages SET tags = %(tags)s WHERE slug = %(slug)s",
                    {'tags': list(new_tags), 'slug': slug},
                )

def plan_overwrite_tags(cur):
    print("** Query plan for Option A overwrite tags")
    cur.execute("EXPLAIN DELETE FROM a__page_tags WHERE page_id = 1")
    print_plan(cur)

    cur.execute("EXPLAIN INSERT INTO a__page_tags (tag, page_id) VALUES ('plan', 1)")
    print_plan(cur)

    print("** Query plan for Option B remove tags")
    cur.execute("EXPLAIN UPDATE b__pages SET tags = '{}' WHERE slug = 'plan'")
    print_plan(cur)

def get_tag_cloud(cur, pages, slugs):
    tag_count = PAGES // 4

    with timer("option A tag cloud"):
        # Arbitrary set of pages that we're getting a tag cloud for.
        # Stand-in for sites.
        random.shuffle(slugs)
        cloud_slugs = slugs[:tag_count]
        cloud_page_ids = [pages[slug].page_id_a for slug in cloud_slugs]

        cur.execute("""
                SELECT
                    DISTINCT(tag),
                    COUNT(tag)
                FROM a__page_tags
                WHERE page_id IN %(page_ids)s
                GROUP BY tag
                ORDER BY tag
            """,
            {'page_ids': tuple(cloud_page_ids)},
        )
        tag_counts = {tag: count for tag, count in cur.fetchall()}

    with timer("option B tag cloud"):
        random.shuffle(slugs)
        cloud_slugs = slugs[:tag_count]
        cloud_page_ids = [pages[slug].page_id_b for slug in cloud_slugs]

        cur.execute(
            "SELECT tags FROM b__pages WHERE page_id IN %(page_ids)s",
            {'page_ids': tuple(cloud_page_ids)},
        )

        tag_counts = defaultdict(lambda: 0)
        for tags, in cur.fetchall():
            for tag in tags:
                tag_counts[tag] += 1

def plan_tag_cloud(cur):
    print("** Query plan for Option A tag cloud")
    cur.execute("""
        EXPLAIN SELECT
            DISTINCT(tag),
            COUNT(tag)
        FROM a__page_tags
        WHERE page_id IN (1, 2)
        GROUP BY tag
        ORDER BY tag
    """)
    print_plan(cur)

    print("** Query plan for Option B tag cloud")
    cur.execute("EXPLAIN SELECT tags FROM b__pages WHERE page_id IN (1, 2)")
    print_plan(cur)

# Main

if __name__ == "__main__":
    print(f"Pages: {PAGES:,}")
    print(f"Operations: {OPERATIONS:,}")
    print()

    with database_connect() as conn:
        print("Setting up database...")
        with conn.cursor() as cur:
            database_schema_create(cur)

        print("Populating pages...")
        with conn.cursor() as cur:
            pages = database_populate(cur)
            slugs = list(pages.keys())

        print("Reading page tags...")
        with conn.cursor() as cur:
            read_tags(cur, pages, slugs)

        print("Adding page tags...")
        with conn.cursor() as cur:
            add_tags(cur, pages, slugs)

        print("Removing page tags...")
        with conn.cursor() as cur:
            remove_tags(cur, pages, slugs)

        print("Changing page tags... (add & remove)")
        with conn.cursor() as cur:
            change_tags(cur, pages, slugs)

        print("Overwriting page tags...")
        with conn.cursor() as cur:
            overwrite_tags(cur, pages, slugs)

        print("Get tag cloud...")
        with conn.cursor() as cur:
            get_tag_cloud(cur, pages, slugs)

        print("Showing query plans...")
        with conn.cursor() as cur:
            plan_database_populate(cur)
            plan_read_tags(cur)
            plan_add_tags(cur)
            plan_remove_tags(cur)
            plan_overwrite_tags(cur)
            plan_tag_cloud(cur)

    print("Finished!")
