#!/usr/bin/env python3
"""Encrypts embedded resources before build.
Run: python encrypt_resources.py
"""
import os
import sys

EMBED_DIR = os.path.join(os.path.dirname(__file__), "embed")

# Scattered key - split across multiple variables in the Rust code
KEY_PARTS = [0x56, 0x31, 0x50, 0x45, 0x72, 0x53, 0x65, 0x72, 0x76, 0x49, 0x63, 0x65]
# XxXx - V 1 P e r S e r v I c e


def encrypt(data: bytes, key: list[int]) -> bytes:
    key_len = len(key)
    return bytes(b ^ key[i % key_len] for i, b in enumerate(data))


def process_file(name: str):
    src = os.path.join(EMBED_DIR, name)
    dst = os.path.join(EMBED_DIR, name + ".enc")
    if not os.path.exists(src):
        print(f"SKIP {name} (not found)")
        return
    with open(src, "rb") as f:
        data = f.read()
    encrypted = encrypt(data, KEY_PARTS)
    with open(dst, "wb") as f:
        f.write(encrypted)
    print(f"OK   {name} ({len(data)} -> {len(encrypted)} bytes)")


if __name__ == "__main__":
    process_file("platform-tools.zip")
    process_file("unisoc.zip")
    print("Done. Encrypted files ready for embedding.")
