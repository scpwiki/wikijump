from dataclasses import dataclass
from datetime import datetime
from typing import Optional


@dataclass(frozen=True)
class User:
    wikidot_id: Optional[int]  # None means the ID isn't known, so we should assign our own
    created_at: datetime
    name: str
    slug: str
    avatar: bytes


@dataclass(frozen=True)
class Site:
    wikidot_id: Optional[int]
    created_at: datetime
    name: str
    slug: str
    subtitle: str
    description: str


@dataclass(frozen=True)
class Page:
    wikidot_id: Optional[int]
    created_at: datetime
    updated_at: datetime
    site_id: int
    title: str
    slug: str
    discussion_thread_id: Optional[int]
