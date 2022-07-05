import hashlib
from binascii import hexlify
from typing import Iterable, Optional, Set, Union

from .constants import *
from .counter import IncrementingCounter
from .structures import *
from .utils import get_page_category

import psycopg2
from cuid import cuid


class Generator:
    """
    Generates SQL and S3 invocations.

    This produces a SQL file to ingest data into DEEPWELL, as well as a
    shells cript which invokes the aws utility to upload data to S3.

    The class also tracks the state of all imported Wikidot data,
    as encountered. This is necessary to avoid inserting
    duplicate data.
    """

    __slots__ = (
        "sql_buffer",
        "sh_buffer",
        "cursor",
        "s3_bucket",
        "page_category_id",
        "user_ids",
        "user_slugs",
        "site_ids",
        "site_slugs",
        "page_ids",
        "page_slugs",
        "page_revision_ids",
        "page_revision_numbers",
        "page_categories",
        "file_names",
        "blob_hashes",
        "text_hashes",
    )

    def __init__(self, sql_buffer, sh_buffer, cursor, s3_bucket, last_page_category_id):
        self.sql_buffer = sql_buffer
        self.sh_buffer = sh_buffer
        self.cursor = cursor
        self.s3_bucket = s3_bucket
        self.page_category_id = IncrementingCounter(last_page_category_id)

        self.user_ids, self.user_slugs = set(), set()  # Set[int], Set[str]
        self.site_ids, self.site_slugs = set(), set()  # Set[int], Set[str]
        self.page_ids, self.page_slugs = set(), set()  # Set[int], Set[Tuple[int, str]]
        self.page_revision_ids = set()  # Set[int]
        self.page_revision_numbers = set()  # Set[Tuple[int, int]]
        self.page_categories = {}  # dict[Tuple[int, str], int]
        self.file_names = {}  # dict[Tuple[int, str], str]
        self.blob_hashes = {}  # dict[bytes, str]
        self.text_hashes = set()  # Set[bytes]

    def format(self, query: str, parameters=()) -> str:
        return self.cursor.mogrify(query, parameters).decode("utf-8")

    def append_sql(self, query: str, parameters=()):
        sql_line = self.format(query, parameters)
        self.sql_buffer.write(f"{sql_line};\n")

    def section_sql(self, name: str):
        self.sql_buffer.write(f"\n\n--\n-- {name}\n--\n\n")

    def append_sh(self, data: bytes, data_hash: bytes):
        def bash_escape(d: bytes) -> str:
            r"""Bash-escape binary strings. e.g. $'\x00'"""

            inner = "".join(f"\\x{b:02x}" for b in d)
            return f"$'{inner}'"

        data_hash_hex = hexlify(data_hash).decode("utf-8")
        bucket_path = f"s3://{self.s3_bucket}/{data_hash_hex}"

        self.sh_buffer.write(
            'file="$(mktemp)"\n'
            f"printf '%s' {bash_escape(data)} > \"$file\"\n"
            f'aws cp "$file" {bucket_path}\n'
            f'rm "$file"\n\n'
        )

        return bucket_path

    def section_sh(self, name: str):
        self.sh_buffer.write(f"\n\n#\n# {name}\n#\n\n")

    def add_user(self, user: User):
        if (
            self.id_exists(self.user_ids, user.wikidot_id)
            or user.slug in self.user_slugs
        ):
            return

        avatar_path = self.add_blob(user.avatar)

        # TODO change over when user table changes, remaining fields
        self.append_sql(
            "INSERT INTO users (id, slug, username, avatar_path, created_at) VALUES (%s, %s, %s, %s)",
            (user.wikidot_id, user.slug, user.name, avatar_path, user.created_at),
        )

        self.id_add(self.user_ids, user.wikidot_id)
        self.user_slugs.add(user.slug)

    def add_site(self, site: Site):
        if (
            self.id_exists(self.site_ids, site.wikidot_id)
            or site.slug in self.site_slugs
        ):
            return

        self.append_sql(
            "INSERT INTO site (site_id, name, slug, subtitle, description) VALUES (%s, %s, %s, %s, %s)",
            (site.wikidot_id, site.name, site.slug, site.subtitle, site.description),
        )

        self.id_add(self.site_ids, site.wikidot_id)
        self.site_slugs.add(site.slug)

    def add_page(self, page: Page):
        if (
            self.id_exists(self.page_ids, page.wikidot_id)
            or (page.site_id, page.slug) in self.page_slugs
        ):
            return

        page_category_id = self.add_page_category(page.site_id, get_page_category(page.slug))
        self.append_sql(
            "INSERT INTO page (page_id, created_at, updated_at, site_id, page_category_id, slug, discussion_thread_id) VALUES (%s, %s, %s, %s, %s, %s, %s)",
            (
                page.wikidot_id,
                page.created_at,
                page.updated_at,
                page.site_id,
                page_category_id,
                page.slug,
                page.discussion_thread_id,
            ),
        )

        self.id_add(self.page_ids, page.wikidot_id)
        self.page_slugs.add((page.site_id, page.slug))

    def add_page_revisions(self, revisions: Iterable[PageRevision]):
        for revision in revisions:
            self.add_page_revision(revision)

    def add_page_revision(self, revision: PageRevision):
        if (
            self.id_exists(self.page_revision_ids, revision.wikidot_id)
            or (revision.page_id, revision.revision_number)
            in self.page_revision_numbers
        ):
            return

        if revision.flags == "N" or revision.revision_number == 0:
            revision_type = "created"
        elif revision.flags == "R":
            revision_type = "move"
        else:
            revision_type = "regular"

        wikitext_hash = self.add_text(revision.wikitext)
        compiled_hash = self.add_text(revision.html)

        # TODO per-revision fields?
        self.append_sql(
            "INSERT INTO page_revision (revision_id, revision_type, revision_number, created_at, page_id, site_id, user_id, wikitext_hash, compiled_hash, compiled_at, compiled_generator, slug, title, tags, comments) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)",
            (
                revision.wikidot_id,
                revision_type,
                revision.revision_number,
                revision.created_at,
                revision.page_id,
                revision.site_id,
                revision.user_id,
                wikitext_hash,
                compiled_hash,
                revision.created_at,
                "Imported from Wikidot",
                revision.slug,
                revision.title,
                revision.tags,
                revision.comments,
            ),
        )

        self.id_add(self.page_revision_ids, revision.wikidot_id)
        self.page_revision_numbers.add((revision.page_id, revision.revision_number))

    def add_page_votes(self, votes: Iterable[PageVote]):
        for vote in votes:
            self.add_page_vote(vote)

    def add_page_vote(self, vote: PageVote):
        self.append_sql(
            "INSERT INTO page_vote (created_at, page_id, user_id, value)",
            (UNKNOWN_CREATION_DATE, vote.page_id, vote.user_id, vote.value),
        )

    def add_page_lock(self, page_id: int, locked: bool = True):
        if locked:
            self.append_sql(
                "INSERT INTO page_lock (created_at, lock_type, page_id, user_id, reason) VALUES (%s, %s, %s, %s, %s)",
                (
                    UNKNOWN_CREATION_DATE,
                    "wikidot",
                    page_id,
                    ANONYMOUS_USER_ID,
                    "Imported from Wikidot",
                ),
            )

    def add_page_category(self, site_id: int, category_slug: str) -> int:
        page_category_id = self.page_categories.get((site_id, category_slug))

        if page_category_id is None:
            page_category_id = self.page_category_id.next()
            self.append_sql(
                "INSERT INTO page_category (category_id, site_id, slug) VALUES (%s, %s, %s)",
                (page_category_id, site_id, category_slug),
            )

        return page_category_id

    def add_file(self, file: File):
        file_id = self.file_names.get((file.page_id, file.name))

        if file_id is None:
            file_id = cuid()
            self.append_sql(
                "INSERT INTO file (file_id, created_at, name, page_id) VALUES (%s, %s, %s, %s)",
                (file_id, file.created_at, file.name, file.page_id),
            )
            self.file_names[(file.page_id, file.name)] = file_id

        return file_id

    # TODO add forums

    def add_blob(self, data: bytes) -> str:
        data_hash = hashlib.sha512(data).digest()
        s3_url = self.blob_hashes.get(data_hash)

        if s3_url is None:
            s3_url = self.append_sh(data, data_hash)
            self.blob_hashes[data_hash] = s3_url

        return s3_url

    def add_text(self, text: str) -> bytes:
        text_bytes = text.encode("utf-8")
        text_hash = hashlib.sha512(text_bytes).digest()

        if text_hash not in self.text_hashes:
            self.append_sql(
                "INSERT INTO text (hash, contents) VALUES (%s, %s)", (text_hash, text)
            )
            self.text_hashes.add(text_hash)

        return text_hash

    def id_exists(self, field: Set[int], id: Optional[int]) -> bool:
        if id is None:
            return False

        return id in field

    def id_add(self, field: Set[int], id: Optional[int]):
        if id is None:
            return

        field.add(id)


def generate_seed(
    runner: callable,
    *,
    sql_path: str,
    sh_path: str,
    s3_bucket: str,
    postgres_url: str,
    last_page_category_id: int = 0,
):
    """
    Given a function which takes a Generator, run through whatever backup and add all the relevant information.
    The generator will ensure duplicate data is not added.
    """

    with open(sql_path, "w") as sql_file:
        with open(sh_path, "w") as sh_file:
            with psycopg2.connect(postgres_url) as connection:
                with connection.cursor() as cursor:
                    generator = Generator(
                        sql_file, sh_file, cursor, s3_bucket, last_page_category_id,
                    )
                    runner(generator)
