from Crypto.Hash import KangarooTwelve

from typing import Optional


def from_js_timestamp(value: Optional[int]) -> Optional[int]:
    if value is None:
        return None
    else:
        return value // 1000


def kangaroo_twelve(input: str) -> str:
    data = input.encode("utf-8")
    hash = KangarooTwelve.new(custom=data)
    return hash.read(26).hex()
