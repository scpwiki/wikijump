#!/usr/bin/python3

import os
import re
import sys

import inflection
import toml

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

# Converting bool string to a Python bool
BOOL_VALUES = {
    "true": True,
    "false": False,
}


# Improve readability of some objects, like sets
def check_format(value):
    if isinstance(value, (set, frozenset)):
        return str(list(map(check_format, value)))
    elif isinstance(value, bool):
        return str(value).lower()

    return str(value)


# Case-insensitivity and converting from kebab-case
def convert_name(value):
    value = inflection.underscore(value)
    value = inflection.camelize(value)
    return value.lower()


# Find all rust files that might have rule definitions
def get_submodule_paths(directory):
    def process(path):
        if not path.endswith(".rs"):
            path = os.path.join(path, "rule.rs")

        return os.path.join(directory, path)

    return map(process, os.listdir(directory))


def load_block_data(blocks_path):
    # Load config
    with open(blocks_path) as file:
        blocks = toml.load(file)
        blocks = {
            convert_name(name): value
            for name, value in blocks.items()
        }

    # Adjust configuration
    # Make implicit fields explicit, etc.
    for name, block in blocks.items():
        # Aliases
        aliases = block.get("aliases", [])
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
    for path in get_submodule_paths(BLOCK_DIRECTORY):
        with open(path) as file:
            contents = file.read()

        for match in BLOCK_RULE_REGEX.finditer(contents):
            name = convert_name(match[1])

            if name in EXCLUDE_BLOCKS:
                continue

            block_rules[name] = {
                "aliases": frozenset(map(convert_name, eval(match[2]))),
                "accepts-star": BOOL_VALUES[match[3]],
                "accepts-score": BOOL_VALUES[match[4]],
                "accepts-newlines": BOOL_VALUES[match[5]],
            }

    return blocks, block_rules


def load_module_data(modules_path):
    # Load blocks
    with open(modules_path) as file:
        modules = toml.load(file)
        modules = {
            convert_name(name): value
            for name, value in modules.items()
        }

    # Adjust configuration
    # Make implicit fields explicit, etc.
    for name, module in modules.items():
        # Aliases
        aliases = module.get("aliases", [])
        aliases.append(name)
        module["aliases"] = frozenset(aliases)

    # Load rules
    module_rules = {}
    for path in get_submodule_paths(MODULE_DIRECTORY):
        with open(path) as file:
            contents = file.read()

        for match in MODULE_RULE_REGEX.finditer(contents):
            name = convert_name(match[1])

            if name in EXCLUDE_MODULES:
                continue

            module_rules[name] = {
                "aliases": frozenset(map(convert_name, eval(match[2]))),
            }

    return modules, module_rules


def compare_block_data(block_conf, block_rules):
    success = True

    # Check for new or removed blocks
    block_conf_names = frozenset(map(convert_name, block_conf.keys()))
    block_rule_names = frozenset(map(convert_name, block_rules.keys()))

    added = block_rule_names - block_conf_names
    deleted = block_conf_names - block_rule_names

    if added:
        print("Added blocks:")

        for name in added:
            print(f"* {name}")

        print()
        success = False

    if deleted:
        print("Deleted blocks")

        for name in deleted:
            print(f"* {name}")

        print()
        success = False

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
                print(f"    Code:   {check_format(rule[key])}")
                print(f"    Config: {check_format(conf[key])}")
                success = False

        check("aliases")
        check("accepts-star")
        check("accepts-score")
        check("accepts-newlines")

    print()
    return success


def compare_module_data(module_conf, module_rules):
    success = True

    # Check for new or removed modules
    module_conf_names = frozenset(module_conf.keys())
    module_rule_names = frozenset(module_rules.keys())

    added = module_rule_names - module_conf_names
    deleted = module_conf_names - module_rule_names

    if added:
        print("Added modules:")

        for name in sorted(added):
            print(f"* {name}")

        print()
        success = False

    if deleted:
        print("Deleted modules:")

        for name in sorted(deleted):
            print(f"* {name}")

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
                print(f"    Code:   {check_format(rule[key])}")
                print(f"    Config: {check_format(conf[key])}")
                success = False

        check("aliases")

    print()
    return success


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: check_blocks.py <ftml-root-dir>")
        sys.exit(-1)

    root_dir = sys.argv[1]
    success = True

    blocks_path = os.path.join(root_dir, "conf/blocks.toml")
    blocks, block_rules = load_block_data(blocks_path)
    success &= compare_block_data(blocks, block_rules)

    modules_path = os.path.join(root_dir, "conf/modules.toml")
    modules, module_rules = load_module_data(modules_path)
    success &= compare_module_data(modules, module_rules)

    if success:
        print("FTML configuration check passed.")
        sys.exit(0)
    else:
        print("FTML configuration check failed!")
        sys.exit(1)
