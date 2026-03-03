#!/usr/bin/env python3
import argparse
import pathlib
import sys
from typing import List, Set


REQUIRED_FILES = [
    "include/miyabi/miyabi.h",
    "include/miyabi/bridge.h",
    "include/miyabi_logic_cxx/lib.h",
    "include/rust/cxx.h",
    "lib/libmiyabi_logic.a",
    "lib/libmiyabi_logic_cxx.a",
    "lib/libmiyabi_runtime.a",
    "lib/libbox2d.a",
    "cmake/MIYABIConfig.cmake",
    "cmake/MIYABIConfigVersion.cmake",
    "template_CMakeLists.txt",
    "examples/main.cpp",
    "docs/SDK_DEFINITION.md",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check MIYABI SDK bundle required artifacts."
    )
    parser.add_argument(
        "--sdk-dir",
        default="sdk",
        help="Path to extracted SDK root directory (default: ./sdk)",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when unexpected files are found in addition to missing files.",
    )
    return parser.parse_args()


def to_rel_posix(path: pathlib.Path, root: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()


def collect_files(root: pathlib.Path) -> Set[str]:
    found: Set[str] = set()
    for item in root.rglob("*"):
        if item.is_file():
            found.add(to_rel_posix(item, root))
    return found


def print_list(title: str, items: List[str]) -> None:
    print(title)
    if not items:
        print("- (none)")
        return
    for item in items:
        print(f"- {item}")


def main() -> int:
    args = parse_args()
    sdk_dir = pathlib.Path(args.sdk_dir).resolve()

    if not sdk_dir.exists() or not sdk_dir.is_dir():
        print(f"ERROR: SDK directory not found: {sdk_dir}", file=sys.stderr)
        return 2

    required = sorted(REQUIRED_FILES)
    required_set = set(required)
    actual_set = collect_files(sdk_dir)

    missing = sorted(required_set - actual_set)
    unexpected = sorted(actual_set - required_set)

    print(f"SDK root: {sdk_dir}")
    print(f"Required files: {len(required)}")
    print(f"Actual files: {len(actual_set)}")
    print("")
    print_list("Missing files", missing)
    print("")
    print_list("Unexpected files", unexpected)

    if missing:
        print("")
        print("RESULT: FAIL (missing required files)")
        return 1

    if args.strict and unexpected:
        print("")
        print("RESULT: FAIL (unexpected files found in --strict mode)")
        return 1

    print("")
    print("RESULT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
