#!/usr/bin/python3

import json
import random
import time
from uuid import uuid4

import psycopg2

TAGS = open('tags.txt').read().splitlines()

# Utilities

class Timer:
    def __init__(self, name):
        self.name = name
        self.start = None

    def __enter__(self):
        self.start = time.monotonic_ns()

    def __exit__(self, type, value, traceback):
        elapsed_ns = time.monotonic_ns() - self.start
        elapsed_ms = elapsed_ns / 1e6
        print(f"Task '{self.name}' in {elapsed_ms:.4f} ms")

def random_tags(min_len=1, max_len=20):
    return {
        random.choice(TAGS)
        for _ in range(random.randint(min_len, max_len))
    }

def generate_slugs_and_tags():
    slugs = {}

    for _ in range(500):
        slug = str(uuid4())
        tags = random_tags()

        slugs[slug] = tags

    return slugs

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

    with Timer("option A populate"):
        for slug, tags in pages.items():
            cur.execute(
                "INSERT INTO a__pages (slug) VALUES (%(slug)s) RETURNING page_id",
                {'slug': slug},
            )
            page_id_a, = cur.fetchone()

            for tag in tags:
                cur.execute(
                    "INSERT INTO a__page_tags (tag, page_id) VALUES (%(tag)s, %(page_id)d)",
                    {'tag': tag, page_id: page_id_a},
                )

            pages[slug] = [tags, page_id_a, None]

    with Timer("option B populate"):
        for slug, tags in pages.items():
            cur.execute(
                "INSERT INTO b__pages (slug, tags) VALUES (%(slug)s, %(tags)s) RETURNING page_id",
                {'slug': slug, 'tags': tags},
            )
            page_id_b, = cur.fetchone()
            pages[slug][2] = page_id_b

    return pages

# Operations

def read_tags(cur, pages):
    ...

# Main

if __name__ == "__main__":
    with database_connect() as conn:
        print("Setting up database...")
        with conn.cursor() as cur:
            database_schema_create(cur)

        print("Populating pages...")
        with conn.cursor() as cur:
            pages = database_populate(cur)

        print("Reading page tags...")
        with conn.cursor() as cur:
            read_tags(cur, pages)

    print("Finished!")
