#!/usr/bin/python3

import os
import re
import sys
from collections import defaultdict

import inflection
import tomllib

EXCLUDE_BLOCKS = ["later"]
EXCLUDE_MODULES = []

BLOCK_DIRECTORY = "src/parsing/rule/impls/block/blocks"
MODULE_DIRECTORY = "src/parsing/rule/impls/block/blocks/module/modules"

# Regular expressions to extract rules from source
BLOCK_RULE_REGEX = re.compile(
    r"""pub const BLOCK_\w+: BlockRule = BlockRule \{
    name: "block-([\w\-]+)",
    accepts_names: &(\[(?:"[^"]+"(?:, )?)+\]),
    accepts_star: (true|false),
    accepts_score: (true|false),
    accepts_newlines: (true|false),
    parse_fn.*
\};"""
)

MODULE_RULE_REGEX = re.compile(
    r"""pub const MODULE_\w+: ModuleRule = ModuleRule \{
    name: "module-([\w\-]+)",
    accepts_names: &(\[(?:"[^"]+"(?:, )?)+\]),
    parse_fn.*
\};"""
)

MODULE_EXAMPLE_REGEX = re.compile(r"\[\[module (\w+).*\]\]")

# Converting bool string to a Python bool
BOOL_VALUES = {
    "true": True,
    "false": False,
}

# Aliases which have one of these prefixes should be ignored because they
# exist because of Wikidot's weird alignment naming or jank.
BLOCK_NAME_IGNORE_PREFIXES = [
    "<",
    "=",
    ">",
    "f<",
    "f>",
    "module654",
]


def check_block_alias_in_doc(alias):
    for prefix in BLOCK_NAME_IGNORE_PREFIXES:
        if alias.startswith(prefix):
            return False
    return True


# Container for primitives which we want by reference
class Container:
    __slots__ = ("data",)

    def __init__(self, data):
        self.data = data

    def get(self):
        return self.data

    def set(self, data):
        self.data = data


# Improve readability of some objects, like sets
def format_check_value(value):
    if isinstance(value, (set, frozenset)):
        return str(list(map(format_check_value, value)))
    elif isinstance(value, bool):
        return str(value).casefold()

    return str(value)


# Case-insensitivity and converting from kebab-case
def convert_block_name(value):
    return value.casefold()


def convert_module_name(value):
    value = inflection.underscore(value)
    value = inflection.camelize(value)
    return value.casefold()


# Find all Rust files that might have rule definitions
def get_submodule_paths(directory):
    def process(path):
        if not path.endswith(".rs"):
            path = os.path.join(path, "rule.rs")

        return os.path.join(directory, path)

    return map(process, os.listdir(directory))


def load_block_data(root_dir):
    blocks_path = os.path.join(root_dir, "conf/blocks.toml")
    block_rules_path = os.path.join(root_dir, BLOCK_DIRECTORY)

    # Load config
    with open(blocks_path, "rb") as file:
        blocks = tomllib.load(file)

        # Normalize module keys for case-insensitive access
        blocks = {convert_block_name(name): value for name, value in blocks.items()}

    # Adjust configuration
    # Make implicit fields explicit, etc.
    for name, block in blocks.items():
        # Aliases
        # We use sets so alias order doesn't matter
        aliases = block.get("aliases", [])

        if not block.get("exclude-name", False):
            aliases.append(name)

        block["aliases"] = frozenset(aliases)

        # Flags
        if "accepts-star" not in block:
            block["accepts-star"] = False

        if "accepts-score" not in block:
            block["accepts-score"] = False

        if "accepts-newlines" not in block:
            block["accepts-newlines"] = False

    # Load rules
    block_rules = {}
    for path in get_submodule_paths(block_rules_path):
        with open(path) as file:
            contents = file.read()

        for match in BLOCK_RULE_REGEX.finditer(contents):
            name = convert_block_name(match[1])

            if name in EXCLUDE_BLOCKS:
                continue

            block_rules[name] = {
                "aliases": frozenset(s.casefold() for s in eval(match[2])),
                "accepts-star": BOOL_VALUES[match[3]],
                "accepts-score": BOOL_VALUES[match[4]],
                "accepts-newlines": BOOL_VALUES[match[5]],
            }

    return blocks, block_rules


def load_module_data(root_dir):
    modules_path = os.path.join(root_dir, "conf/modules.toml")
    module_rules_path = os.path.join(root_dir, MODULE_DIRECTORY)

    # Load blocks
    with open(modules_path, "rb") as file:
        modules = tomllib.load(file)

        # Normalize module keys for case-insensitive access
        modules = {convert_module_name(name): value for name, value in modules.items()}

    # Adjust configuration
    # Make implicit fields explicit, etc.
    for name, module in modules.items():
        # Aliases
        # We use sets so alias order doesn't matter
        aliases = module.get("aliases", [])
        aliases.append(name)
        module["aliases"] = frozenset(aliases)

    # Load rules
    module_rules = {}
    for path in get_submodule_paths(module_rules_path):
        with open(path) as file:
            contents = file.read()

        for match in MODULE_RULE_REGEX.finditer(contents):
            name = convert_module_name(match[1])

            if name in EXCLUDE_MODULES:
                continue

            module_rules[name] = {
                "aliases": frozenset(s.casefold() for s in eval(match[2])),
            }

    return modules, module_rules


# Load documentation files
def load_block_docs(root_dir):
    blocks_path = os.path.join(root_dir, "docs/Blocks.md")

    with open(blocks_path) as file:
        return file.read()


def load_module_docs(root_dir):
    blocks_path = os.path.join(root_dir, "docs/Modules.md")

    with open(blocks_path) as file:
        return file.read()


# Compare extracted data
def compare_block_data(block_conf, block_rules):
    success = Container(True)

    # Check for new or removed blocks
    block_conf_names = frozenset(map(convert_block_name, block_conf.keys()))
    block_rule_names = frozenset(map(convert_block_name, block_rules.keys()))

    added = block_rule_names - block_conf_names
    deleted = block_conf_names - block_rule_names

    if added:
        print("!! Added blocks !!")

        for name in added:
            print(f"- {name}")

        print()
        success.set(False)

    if deleted:
        print("!! Deleted blocks !!")

        for name in deleted:
            print(f"- {name}")

        print()
        success.set(False)

    # Check contents of each block
    print("Checking blocks:")
    for name in sorted(block_rules.keys()):
        if name not in block_conf:
            print(f"! {name} (MISSING)")
            continue

        # Check block
        print(f"+ {name}")
        conf = block_conf[name]
        rule = block_rules[name]

        def check(key):
            if rule[key] != conf[key]:
                print(f"  Key {key} differs!")
                print(f"    Code:   {format_check_value(rule[key])}")
                print(f"    Config: {format_check_value(conf[key])}")
                success.set(False)

        check("aliases")
        check("accepts-star")
        check("accepts-score")
        check("accepts-newlines")

    print()
    return success.get()


def compare_module_data(module_conf, module_rules):
    success = True

    # Check for new or removed modules
    module_conf_names = frozenset(module_conf.keys())
    module_rule_names = frozenset(module_rules.keys())

    added = module_rule_names - module_conf_names
    deleted = module_conf_names - module_rule_names

    if added:
        print("!! Added modules !!")

        for name in sorted(added):
            print(f"- {name}")

        print()
        success = False

    if deleted:
        print("!! Deleted modules !!")

        for name in sorted(deleted):
            print(f"- {name}")

        print()
        success = False

    # Check contents of each module
    print("Checking modules:")
    for name in sorted(module_rules.keys()):
        if name not in module_conf:
            print(f"! {name} (MISSING)")
            continue

        # Check module
        print(f"+ {name}")
        conf = module_conf[name]
        rule = module_rules[name]

        def check(key):
            if rule[key] != conf[key]:
                print(f"  Key {key}:")
                print(f"    Code:   {format_check_value(rule[key])}")
                print(f"    Config: {format_check_value(conf[key])}")
                success = False

        check("aliases")

    print()
    return success


# Check documentation files
#
# I am not writing a full markdown scraper, besides
# documentation is loose to explain things and can't
# capture everything well anyways.
#
# Instead, this will look for the presence of the blocks'
# aliases, and if any are missing, the human knows to
# go through and ensure everything is present.
def check_block_docs(block_conf, block_docs):
    missing_aliases = defaultdict(list)

    for name, block in block_conf.items():
        for alias in filter(check_block_alias_in_doc, block["aliases"]):
            if f"`{alias}`" not in block_docs:
                missing_aliases[name].append(alias)

    if missing_aliases:
        print("!! Missing documentation for blocks !!")

        for name in sorted(missing_aliases.keys()):
            aliases = missing_aliases[name]
            aliases.sort()
            for alias in aliases:
                print(f"- {alias}")

        print()

    return not missing_aliases


def check_module_docs(module_conf, module_docs):
    def case_insensitive_set(collection):
        return frozenset(s.casefold() for s in collection)

    success = True

    modules_found = case_insensitive_set(MODULE_EXAMPLE_REGEX.findall(module_docs))
    modules_expected = case_insensitive_set(module_conf.keys())

    added = modules_found - modules_expected
    deleted = modules_expected - modules_found

    if added:
        print("!! Added module documentation !!")

        for name in added:
            print(f"- {name}")

        print()
        success = False

    if deleted:
        print("!! Deleted module documentation !!")

        for name in deleted:
            print(f"- {name}")

        print()
        success = False

    return success


if __name__ == "__main__":
    if len(sys.argv) >= 3:
        print("Usage: check_blocks.py <ftml-root-dir>")
        sys.exit(-1)

    success = True
    if len(sys.argv) == 2:
        root_dir = sys.argv[1]
    else:
        root_dir = "."

    blocks, block_rules = load_block_data(root_dir)
    block_docs = load_block_docs(root_dir)

    success &= compare_block_data(blocks, block_rules)
    success &= check_block_docs(blocks, block_docs)

    modules, module_rules = load_module_data(root_dir)
    module_docs = load_module_docs(root_dir)

    success &= compare_module_data(modules, module_rules)
    success &= check_module_docs(modules, module_docs)

    if success:
        print("FTML configuration check passed.")
        sys.exit(0)
    else:
        print("FTML configuration check failed!")
        sys.exit(1)
