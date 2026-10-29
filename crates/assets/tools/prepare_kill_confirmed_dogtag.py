#!/usr/bin/env python3
"""Build the Kill Confirmed dog-tag mesh and team-tinted textures.

The checked-in output is intentionally engine data rather than a runtime glTF
dependency. The source model is CC-BY-4.0; see the adjacent ATTRIBUTION.md.
"""

from __future__ import annotations

import argparse
import json
import struct
from pathlib import Path

import numpy as np
from PIL import Image


GLB_MAGIC = b"glTF"
JSON_CHUNK = 0x4E4F534A
BIN_CHUNK = 0x004E4942
OUTPUT_MAGIC = b"IW4LKC1\0"
TARGET_MAX_EXTENT = 14.4
TEAM_TINTS = {
    "friendly": np.array([0.30, 0.62, 1.00], dtype=np.float32),
    "enemy": np.array([1.00, 0.52, 0.18], dtype=np.float32),
}


def read_glb(path: Path) -> tuple[dict, bytes]:
    data = path.read_bytes()
    magic, version, total_length = struct.unpack_from("<4sII", data)
    if magic != GLB_MAGIC or version != 2:
        raise ValueError(f"{path}: expected a GLB 2.0 file")
    if total_length != len(data):
        raise ValueError(f"{path}: GLB length header does not match file size")

    chunks = {}
    offset = 12
    while offset < len(data):
        length, kind = struct.unpack_from("<II", data, offset)
        offset += 8
        chunks[kind] = data[offset : offset + length]
        offset += length
    if JSON_CHUNK not in chunks or BIN_CHUNK not in chunks:
        raise ValueError(f"{path}: missing JSON or binary chunk")
    return json.loads(chunks[JSON_CHUNK].rstrip(b" \0")), chunks[BIN_CHUNK]


def node_local_matrix(node: dict) -> np.ndarray:
    if "matrix" in node:
        return np.asarray(node["matrix"], dtype=np.float64).reshape(4, 4).T
    matrix = np.eye(4, dtype=np.float64)
    if "translation" in node:
        matrix[:3, 3] = node["translation"]
    if "rotation" in node:
        x, y, z, w = node["rotation"]
        matrix[:3, :3] = np.array(
            [
                [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
            ],
            dtype=np.float64,
        )
    if "scale" in node:
        matrix[:3, :3] = matrix[:3, :3] @ np.diag(node["scale"])
    return matrix


def scene_node_transforms(gltf: dict) -> dict[int, np.ndarray]:
    transforms = {}
    nodes = gltf.get("nodes", [])
    scene = gltf.get("scenes", [{}])[gltf.get("scene", 0)]

    def visit(index: int, parent: np.ndarray) -> None:
        world = parent @ node_local_matrix(nodes[index])
        transforms[index] = world
        for child in nodes[index].get("children", []):
            visit(child, world)

    for root in scene.get("nodes", []):
        visit(root, np.eye(4, dtype=np.float64))
    return transforms


def read_accessor(gltf: dict, binary: bytes, index: int) -> np.ndarray:
    accessor = gltf["accessors"][index]
    view = gltf["bufferViews"][accessor["bufferView"]]
    component_dtype = {
        5121: np.dtype("<u1"),
        5123: np.dtype("<u2"),
        5125: np.dtype("<u4"),
        5126: np.dtype("<f4"),
    }[accessor["componentType"]]
    width = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}[accessor["type"]]
    dtype = np.dtype((component_dtype, (width,))) if width > 1 else component_dtype
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride", component_dtype.itemsize * width)
    rows = np.ndarray(
        shape=(accessor["count"],),
        dtype=dtype,
        buffer=binary,
        offset=start,
        strides=(stride,),
    )
    return rows.copy()


def read_mesh(
    gltf: dict, binary: bytes
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    transforms = scene_node_transforms(gltf)
    position_rows = []
    normal_rows = []
    tangent_rows = []
    uv_rows = []
    index_rows = []
    vertex_base = 0

    for node_index, node in enumerate(gltf.get("nodes", [])):
        if "mesh" not in node:
            continue
        world = transforms.get(node_index, np.eye(4, dtype=np.float64))
        normal_matrix = np.linalg.inv(world[:3, :3]).T
        for primitive in gltf["meshes"][node["mesh"]]["primitives"]:
            if primitive.get("mode", 4) != 4:
                raise ValueError("only triangle-list primitives are supported")
            attributes = primitive["attributes"]
            positions = read_accessor(gltf, binary, attributes["POSITION"])
            positions_h = np.concatenate(
                [positions, np.ones((positions.shape[0], 1), dtype=positions.dtype)], axis=1
            )
            positions = (positions_h @ world.T)[:, :3]
            normals = read_accessor(gltf, binary, attributes["NORMAL"])
            normals = normals @ normal_matrix.T
            normals /= np.maximum(np.linalg.norm(normals, axis=1, keepdims=True), 1e-8)
            tangents = read_accessor(gltf, binary, attributes["TANGENT"])[:, :3]
            tangents = tangents @ normal_matrix.T
            tangents /= np.maximum(np.linalg.norm(tangents, axis=1, keepdims=True), 1e-8)
            uvs = read_accessor(gltf, binary, attributes["TEXCOORD_0"])
            indices = read_accessor(gltf, binary, primitive["indices"]).reshape(-1)
            if indices.size % 3 != 0:
                raise ValueError("primitive index count is not triangular")
            position_rows.append(positions)
            normal_rows.append(normals)
            tangent_rows.append(tangents)
            uv_rows.append(uvs)
            index_rows.append(indices.astype(np.uint32) + vertex_base)
            vertex_base += positions.shape[0]

    if not position_rows:
        raise ValueError("GLB contains no mesh primitives")
    return (
        np.concatenate(position_rows),
        np.concatenate(normal_rows),
        np.concatenate(tangent_rows),
        np.concatenate(uv_rows),
    ), np.concatenate(index_rows), transforms


def embed_texture(gltf: dict, binary: bytes, image_index: int) -> Image.Image:
    image = gltf["images"][image_index]
    view = gltf["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    end = start + view["byteLength"]
    from io import BytesIO

    return Image.open(BytesIO(binary[start:end])).convert("RGBA")


def tint_texture(source: Image.Image, tint: np.ndarray) -> Image.Image:
    rgba = np.asarray(source, dtype=np.float32) / 255.0
    luminance = rgba[..., :3] @ np.array([0.2126, 0.7152, 0.0722], dtype=np.float32)
    lit = np.clip(0.10 + luminance * 1.28, 0.0, 1.0)
    rgb = np.clip(lit[..., None] * tint[None, None, :], 0.0, 1.0)
    alpha = np.maximum(rgba[..., 3:4], (luminance > 0.02)[..., None].astype(np.float32))
    return Image.fromarray(np.round(np.concatenate([rgb, alpha], axis=2) * 255.0).astype(np.uint8), "RGBA")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path, help="dog_tags_-_military.glb")
    parser.add_argument("output", type=Path, help="output directory")
    args = parser.parse_args()

    gltf, binary = read_glb(args.source)
    (positions, normals, tangents, uvs), indices, _ = read_mesh(gltf, binary)

    # The Sketchfab scene converts its Z-up authoring frame to glTF Y-up. The
    # renderer wants Z-up, so import the scene and rotate Y-up back to Z-up.
    rotate_z_up = np.array(
        [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]], dtype=np.float64
    )
    positions = positions @ rotate_z_up.T
    normals = normals @ rotate_z_up.T
    normals /= np.maximum(np.linalg.norm(normals, axis=1, keepdims=True), 1e-8)
    tangents = tangents @ rotate_z_up.T
    tangents /= np.maximum(np.linalg.norm(tangents, axis=1, keepdims=True), 1e-8)

    minimum = positions.min(axis=0)
    maximum = positions.max(axis=0)
    center = (minimum + maximum) * 0.5
    extent = float(np.max(maximum - minimum))
    scale = TARGET_MAX_EXTENT / extent
    positions = (positions - center) * scale

    args.output.mkdir(parents=True, exist_ok=True)
    with (args.output / "mesh.bin").open("wb") as output:
        output.write(OUTPUT_MAGIC)
        output.write(struct.pack("<II", positions.shape[0], indices.size))
        for position, normal, tangent, uv in zip(
            positions, normals, tangents, uvs, strict=True
        ):
            output.write(struct.pack("<11f", *position, *normal, *tangent, *uv))
        output.write(indices.astype("<u4", copy=False).tobytes())

    base = embed_texture(gltf, binary, 0)
    for team, tint in TEAM_TINTS.items():
        tint_texture(base, tint).save(args.output / f"{team}.png", optimize=True)

    print(
        f"wrote {positions.shape[0]} vertices, {indices.size // 3} triangles, "
        f"bounds {np.round(maximum - minimum, 3).tolist()}, scale {scale:.5f}"
    )


if __name__ == "__main__":
    main()
