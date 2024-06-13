from typing import Optional


def from_js_timestamp(value: Optional[int]) -> Optional[int]:
    if value is None:
        return None
    else:
        return value // 1000
