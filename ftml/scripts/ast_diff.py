#!/usr/bin/env python3

import subprocess
import sys
from tempfile import NamedTemporaryFile


def split_input(file):
    expected_lines = []
    actual_lines = []
    current = None

    for line in file.readlines():
        if line.startswith("Expected: "):
            current = "expected"
        elif line.startswith("Actual: "):
            current = "actual"

        if current == "expected":
            expected_lines.append(line)
        elif current == "actual":
            actual_lines.append(line)

    expected = "".join(expected_lines)
    actual = "".join(actual_lines)

    return expected, actual


def write_temp_file(contents):
    file = NamedTemporaryFile()
    file.write(contents.encode("utf-8"))
    return file

if __name__ == "__main__":
    if len(sys.argv) < 2:
        expected, actual = split_input(sys.stdin)
    else:
        with open(sys.argv[1]) as file:
            expected, actual = split_input(file)

    expected_file = write_temp_file(expected)
    actual_file = write_temp_file(actual)

    subprocess.run(["diff", "--color", "--", expected_file.name, actual_file.name])
