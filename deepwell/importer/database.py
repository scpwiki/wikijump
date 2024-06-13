import os
import sqlite3


class Database:
    __slots__ = ("conn",)

    def __init__(self, db_url: str) -> None:
        self.conn = sqlite3.connect(db_url)

    def seed(self) -> None:
        seed_path = os.path.join(os.path.dirname(__file__), "seed.sql")

        with open(seed_path) as file:
            self.conn.executescript(file.read())

    def close(self) -> None:
        self.conn.close()
