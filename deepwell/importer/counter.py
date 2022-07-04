from dataclasses import dataclass


@dataclass
class IncrementingCounter:
    value: int = 0

    def next(self):
        self.value += 1
