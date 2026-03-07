#!/usr/bin/env python3
import argparse
import pathlib
import re
import sys
from dataclasses import dataclass
from typing import List, Sequence


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_OBJ_PATHS = ["assets/meshes/arena_cube.obj"]
DEFAULT_TEXTURE_PATHS = ["assets/player.png", "assets/test.png"]
DEFAULT_SHADER_PATHS = [
    "core/src/shaders/lit_textured.vert",
    "core/src/shaders/lit_textured.frag",
]


@dataclass
class CheckMessage:
    level: str
    category: str
    target: str
    detail: str

    def render(self) -> str:
        return f"[{self.level}] {self.category}: {self.target} - {self.detail}"


@dataclass
class ValidationSummary:
    messages: List[CheckMessage]

    @property
    def failures(self) -> List[CheckMessage]:
        return [message for message in self.messages if message.level == "FAIL"]

    def passed(self) -> bool:
        return not self.failures


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate MIYABI 3D mesh/texture/material asset prerequisites."
    )
    parser.add_argument(
        "--root",
        default=str(REPO_ROOT),
        help="Repository root to validate (default: script parent repo root)",
    )
    parser.add_argument(
        "--obj",
        action="append",
        default=[],
        help=(
            "Additional OBJ mesh path to validate. Relative paths are resolved "
            "from --root."
        ),
    )
    parser.add_argument(
        "--texture",
        action="append",
        default=[],
        help=(
            "Additional texture path to validate. Relative paths are resolved "
            "from --root."
        ),
    )
    parser.add_argument(
        "--shader",
        action="append",
        default=[],
        help=(
            "Additional shader path to validate. Relative paths are resolved "
            "from --root."
        ),
    )
    return parser.parse_args()


def make_path(root: pathlib.Path, raw_path: str) -> pathlib.Path:
    path = pathlib.Path(raw_path)
    if path.is_absolute():
        return path
    return root / path


def to_display_path(path: pathlib.Path, root: pathlib.Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def resolve_obj_index(index: int, available_count: int) -> int:
    if index == 0:
        raise ValueError("index 0 is invalid in OBJ")
    if index > 0:
        resolved = index - 1
    else:
        resolved = available_count + index
    if resolved < 0 or resolved >= available_count:
        raise ValueError(f"index {index} is out of range for count={available_count}")
    return resolved


def parse_face_vertex(token: str) -> tuple[int, int | None, int | None]:
    parts = token.split("/")
    if len(parts) > 3:
        raise ValueError(f"malformed face vertex token '{token}'")

    if not parts[0]:
        raise ValueError(f"missing position index in token '{token}'")

    position_index = int(parts[0])
    texcoord_index = None
    normal_index = None

    if len(parts) >= 2 and parts[1]:
        texcoord_index = int(parts[1])
    if len(parts) == 3 and parts[2]:
        normal_index = int(parts[2])

    return position_index, texcoord_index, normal_index


def validate_obj_file(path: pathlib.Path, root: pathlib.Path) -> List[CheckMessage]:
    display_path = to_display_path(path, root)
    messages: List[CheckMessage] = []

    if not path.is_file():
        return [
            CheckMessage(
                "FAIL",
                "mesh",
                display_path,
                "file not found",
            )
        ]

    positions = 0
    texcoords = 0
    normals = 0
    faces = 0
    triangles = 0

    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError as exc:
        return [
            CheckMessage(
                "FAIL",
                "mesh",
                display_path,
                f"failed to read UTF-8 text: {exc}",
            )
        ]

    for line_number, raw_line in enumerate(lines, start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue

        tokens = line.split()
        prefix = tokens[0]

        try:
            if prefix == "v":
                if len(tokens) < 4:
                    raise ValueError("vertex requires 3 coordinates")
                float(tokens[1])
                float(tokens[2])
                float(tokens[3])
                positions += 1
                continue

            if prefix == "vt":
                if len(tokens) < 3:
                    raise ValueError("texcoord requires at least 2 coordinates")
                float(tokens[1])
                float(tokens[2])
                texcoords += 1
                continue

            if prefix == "vn":
                if len(tokens) < 4:
                    raise ValueError("normal requires 3 coordinates")
                float(tokens[1])
                float(tokens[2])
                float(tokens[3])
                normals += 1
                continue

            if prefix != "f":
                continue

            if len(tokens) < 4:
                raise ValueError("face requires at least 3 vertices")

            for token in tokens[1:]:
                position_index, texcoord_index, normal_index = parse_face_vertex(token)
                resolve_obj_index(position_index, positions)
                if texcoord_index is not None:
                    resolve_obj_index(texcoord_index, texcoords)
                if normal_index is not None:
                    resolve_obj_index(normal_index, normals)

            faces += 1
            triangles += len(tokens) - 3
        except ValueError as exc:
            messages.append(
                CheckMessage(
                    "FAIL",
                    "mesh",
                    display_path,
                    f"line {line_number}: {exc}",
                )
            )
            return messages

    if positions == 0:
        return [
            CheckMessage(
                "FAIL",
                "mesh",
                display_path,
                "no vertex positions found",
            )
        ]

    if faces == 0 or triangles == 0:
        return [
            CheckMessage(
                "FAIL",
                "mesh",
                display_path,
                "no drawable faces found",
            )
        ]

    messages.append(
        CheckMessage(
            "PASS",
            "mesh",
            display_path,
            (
                "parseable OBJ "
                f"(positions={positions}, texcoords={texcoords}, "
                f"normals={normals}, faces={faces}, triangles={triangles})"
            ),
        )
    )
    return messages


def validate_required_file(
    category: str,
    path: pathlib.Path,
    root: pathlib.Path,
) -> CheckMessage:
    display_path = to_display_path(path, root)
    if not path.is_file():
        return CheckMessage("FAIL", category, display_path, "file not found")
    return CheckMessage("PASS", category, display_path, "file exists")


def extract_numeric_constant(
    path: pathlib.Path,
    pattern: str,
    label: str,
) -> int:
    content = path.read_text(encoding="utf-8")
    match = re.search(pattern, content, flags=re.MULTILINE)
    if not match:
        raise ValueError(f"{label} not found in {path}")
    return int(match.group(1))


def validate_contracts(root: pathlib.Path) -> List[CheckMessage]:
    logic_path = root / "logic/src/lib.rs"
    core_path = root / "core/src/main.cpp"
    messages: List[CheckMessage] = []

    try:
        logic_mesh_id = extract_numeric_constant(
            logic_path,
            r"const MESH_ID_ARENA_CUBE_3D: u32 = (\d+);",
            "logic mesh id",
        )
        core_mesh_id = extract_numeric_constant(
            core_path,
            r"const uint32_t ARENA_CUBE_MESH_ID = (\d+);",
            "core mesh id",
        )
        if logic_mesh_id != core_mesh_id:
            messages.append(
                CheckMessage(
                    "FAIL",
                    "contract",
                    "arena cube mesh id",
                    (
                        "logic/src/lib.rs and core/src/main.cpp disagree "
                        f"({logic_mesh_id} != {core_mesh_id})"
                    ),
                )
            )
        else:
            messages.append(
                CheckMessage(
                    "PASS",
                    "contract",
                    "arena cube mesh id",
                    f"logic/core agree on {logic_mesh_id}",
                )
            )

        logic_material_id = extract_numeric_constant(
            logic_path,
            r"const MATERIAL_ID_LIT_TEXTURED_3D: u32 = (\d+);",
            "logic material id",
        )
        core_material_id = extract_numeric_constant(
            core_path,
            r"const uint32_t MATERIAL_ID_LIT_TEXTURED_3D = (\d+);",
            "core material id",
        )
        if logic_material_id != core_material_id:
            messages.append(
                CheckMessage(
                    "FAIL",
                    "contract",
                    "lit textured 3D material id",
                    (
                        "logic/src/lib.rs and core/src/main.cpp disagree "
                        f"({logic_material_id} != {core_material_id})"
                    ),
                )
            )
        else:
            messages.append(
                CheckMessage(
                    "PASS",
                    "contract",
                    "lit textured 3D material id",
                    f"logic/core agree on {logic_material_id}",
                )
            )
    except (OSError, UnicodeDecodeError, ValueError) as exc:
        messages.append(CheckMessage("FAIL", "contract", "source constants", str(exc)))

    return messages


def validate_project(
    root: pathlib.Path,
    obj_paths: Sequence[str],
    texture_paths: Sequence[str],
    shader_paths: Sequence[str],
) -> ValidationSummary:
    messages: List[CheckMessage] = []

    for shader_path in DEFAULT_SHADER_PATHS:
        messages.append(validate_required_file("shader", make_path(root, shader_path), root))
    for shader_path in shader_paths:
        messages.append(validate_required_file("shader", make_path(root, shader_path), root))

    for texture_path in DEFAULT_TEXTURE_PATHS:
        messages.append(validate_required_file("texture", make_path(root, texture_path), root))
    for texture_path in texture_paths:
        messages.append(validate_required_file("texture", make_path(root, texture_path), root))

    for obj_path in DEFAULT_OBJ_PATHS:
        messages.extend(validate_obj_file(make_path(root, obj_path), root))
    for obj_path in obj_paths:
        messages.extend(validate_obj_file(make_path(root, obj_path), root))

    messages.extend(validate_contracts(root))
    return ValidationSummary(messages)


def print_summary(summary: ValidationSummary, root: pathlib.Path) -> None:
    print("# MIYABI 3D Asset Validation")
    print(f"Root: {root}")
    print("")
    print("## Checks")
    for message in summary.messages:
        print(f"- {message.render()}")
    print("")
    print("## Summary")
    print(f"- total: {len(summary.messages)}")
    print(f"- pass: {sum(message.level == 'PASS' for message in summary.messages)}")
    print(f"- fail: {len(summary.failures)}")
    if summary.passed():
        print("")
        print("RESULT: PASS")
    else:
        print("")
        print(f"RESULT: FAIL ({len(summary.failures)} errors)")


def main() -> int:
    args = parse_args()
    root = pathlib.Path(args.root).resolve()
    summary = validate_project(root, args.obj, args.texture, args.shader)
    print_summary(summary, root)
    return 0 if summary.passed() else 1


if __name__ == "__main__":
    sys.exit(main())
