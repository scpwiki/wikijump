from dataclasses import dataclass


@dataclass
class IncrementingCounter:
    value: int = 0

    def next(self) -> int:
        self.value += 1
        return value
