#!/usr/bin/python3

import os
import re
import sys

import toml

BLOCK_RULE_REGEX = re.compile(r"""pub const BLOCK_\w+: BlockRule = BlockRule \{
    name: "block-(\w+)",
    accepts_names: &(\[(?:"[^"]+"(?:, )?)+\]),
    accepts_star: (true|false),
    accepts_score: (true|false),
    accepts_newlines: (true|false),
    parse_fn.*
\};""")

MODULE_RULE_REGEX = re.compile(r"""pub const MODULE_\w+: ModuleRule = ModuleRule \{
    name: "module-(\w+)",
    accepts_names: &(\[(?:"[^"]+"(?:, )?)+\]),
    parse_fn.*
\};""")

BOOL_VALUES = {
    "true": True,
    "false": False,
}


def check_format(value):
    if isinstance(value, (set, frozenset)):
        return str(list(value))

    return str(value)


def get_submodule_paths(directory):
    def process(path):
        if not path.endswith(".rs"):
            path = os.path.join(path, "rule.rs")

        return os.path.join(directory, path)

    return map(process, os.listdir(directory))


def load_block_data(blocks_path):
    with open(blocks_path) as file:
        blocks = toml.load(file)

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

    block_rules = {}
    for path in get_submodule_paths("src/parsing/rule/impls/block/blocks"):
        with open(path) as file:
            contents = file.read()

        for match in BLOCK_RULE_REGEX.finditer(contents):
            name = match[1]

            block_rules[name] = {
                "aliases": frozenset(eval(match[2])),
                "accepts-star": BOOL_VALUES[match[3]],
                "accepts-score": BOOL_VALUES[match[4]],
                "accepts-newlines": BOOL_VALUES[match[5]],
            }

    return blocks, block_rules


def load_module_data(modules_path):
    with open(modules_path) as file:
        modules = toml.load(file)

    # Adjust configuration
    # Make implicit fields explicit, etc.
    for name, module in modules.items():
        # Aliases
        aliases = module.get("aliases", [])
        aliases.append(name)
        module["aliases"] = frozenset(aliases)

    module_rules = {}
    for path in get_submodule_paths("src/parsing/rule/impls/block/blocks/module/modules"):
        with open(path) as file:
            contents = file.read()

        for match in MODULE_RULE_REGEX.finditer(contents):
            name = match[1]
            module_rules[name] = {
                "aliases": frozenset(eval(match[2])),
            }

    return modules, module_rules


def compare_block_data(block_conf, block_rules):
    success = True

    # Check for new or removed blocks
    block_conf_names = frozenset(block_conf.keys())
    block_rule_names = frozenset(block_rules.keys())

    added = block_rule_names - block_conf_names
    deleted = block_conf_names - block_rule_names

    if added:
        print("Blocks were added to code, but not to configurations:")

        for name in added:
            print(f"* {name}")

        print()
        success = False

    if deleted:
        print("Blocks were deleted from code, but not from configurations:")

        for name in deleted:
            print(f"* {name}")

        print()
        success = False

    # Check for contents of each block
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
                print(f"  Key {key}:")
                print(f"    Code:   {check_format(rule[key])}")
                print(f"    Config: {check_format(conf[key])}")
                success = False

        check("aliases")
        check("accepts-star")
        check("accepts-score")
        check("accepts-newlines")

    return success


def compare_module_data(module_conf, module_rules):
    print(module_conf)
    print(module_rules)
    return True


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

    if not success:
        print("FTML configuration check failed!")
        sys.exit(1)
