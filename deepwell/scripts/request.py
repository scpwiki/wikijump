#!/usr/bin/env python3

import argparse
import json
import os
import sys

import requests


def color_settings(value):
    match value:
        case "auto":
            fd = sys.stdout.fileno()
            return os.isatty(fd)
        case "always":
            return True
        case "never":
            return False


def parse_data(value):
    try:
        return json.loads(value)
    except json.decoder.JSONDecodeError:
        # Just interpret as a string
        return value


def print_data(data):
    if isinstance(data, str):
        print(data)
    else:
        # Only print on multiple lines if it's "large"
        output = json.dumps(data)
        if len(output) > 16:
            output = json.dumps(data, indent=4)
            print()
        print(output)


def deepwell_request(endpoint, method, data, id=0, color=False):
    r = requests.post(
        endpoint,
        json={
            "jsonrpc": "2.0",
            "method": method,
            "params": data,
            "id": id,
        },
    )

    if color:
        green_start = "\x1b[32m"
        red_start = "\x1b[31m"
        color_end = "\x1b[0m"
    else:
        green_start = ""
        red_start = ""
        color_end = ""

    match r.json():
        case {"jsonrpc": "2.0", "id": id, "result": data}:
            print(f"{green_start}OK  {color_end}", end="")
            print_data(data)
            return 0
        case {"jsonrpc": "2.0", "id": id, "error": data}:
            print(f"{red_start}ERR {color_end}", end="")
            print_data(data)
            return 1


if __name__ == "__main__":
    argparser = argparse.ArgumentParser(
        "deepwell-request",
        description="Helper script to run DEEPWELL JSONRPC requests",
    )
    argparser.add_argument(
        "-H",
        "--host",
        default="localhost",
    )
    argparser.add_argument(
        "-p",
        "--port",
        type=int,
        default=2747,
    )
    argparser.add_argument(
        "-s",
        "--https",
        dest="scheme",
        action="store_const",
        const="https",
        default="http",
    )
    argparser.add_argument(
        "-I",
        "--id",
        default=0,
    )
    argparser.add_argument(
        "-C",
        "--color",
        choices=["never", "auto", "always"],
        default="auto",
    )
    argparser.add_argument("method")
    argparser.add_argument("data", nargs="?", type=parse_data, default="{}")
    args = argparser.parse_args()
    enable_color = color_settings(args.color)

    endpoint = f"{args.scheme}://{args.host}:{args.port}/jsonrpc"
    exit_code = deepwell_request(
        endpoint,
        args.method,
        args.data,
        args.id,
        color=enable_color,
    )

    sys.exit(exit_code)
