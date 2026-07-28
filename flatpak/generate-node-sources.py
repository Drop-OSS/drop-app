#!/usr/bin/env python3
"""Parse pnpm-lock.yaml and generate flatpak node-sources.json.

Outputs entries in the same format as flatpak-node-generator npm mode:
  {
    "type": "file",
    "url": "https://registry.npmjs.org/.../-/...tgz",
    "sha512": "hex",
    "dest": "flatpak-node/npmcache/...",
  }
"""
import json
import sys
import base64
from pathlib import Path

try:
    import yaml
except ImportError:
    print("PyYAML is required. Install with: pip3 install pyyaml", file=sys.stderr)
    sys.exit(1)


def sri_to_hex(sri: str) -> str:
    """Convert SRI hash (e.g. 'sha512-<base64>' or 'sha256-<base64>') to hex string."""
    algo, b64 = sri.split("-", 1)
    raw = base64.b64decode(b64)
    return raw.hex()


def make_npm_url(pkg_name: str, version: str) -> str:
    """Construct npm registry tarball download URL."""
    if pkg_name.startswith("@"):
        scope, name = pkg_name[1:].split("/", 1)
        return f"https://registry.npmjs.org/@{scope}%2f{name}/-/{name}-{version}.tgz"
    else:
        return f"https://registry.npmjs.org/{pkg_name}/-/{pkg_name}-{version}.tgz"


def make_dest(pkg_name: str, version: str) -> str:
    """Construct flatpak vendor destination directory."""
    safe_name = pkg_name.replace("/", "/")
    return f"flatpak-node/npmcache/{safe_name}/{version}"


def main():
    lockfile_path = Path("pnpm-lock.yaml")
    if not lockfile_path.exists():
        print("pnpm-lock.yaml not found", file=sys.stderr)
        sys.exit(1)

    with open(lockfile_path) as f:
        data = yaml.safe_load(f)

    packages = data.get("packages", {})
    if not packages:
        print("No packages section found in pnpm-lock.yaml", file=sys.stderr)
        sys.exit(1)

    entries = []
    seen = set()

    for pkg_key in packages:
        pkg_info = packages[pkg_key]

        # Skip: only generate for published npm packages (not workspace/links)
        resolution = pkg_info.get("resolution")
        if not resolution or "integrity" not in resolution:
            continue

        integrity = resolution["integrity"]

        # Parse key: '@scope/name@version' or 'name@version'
        # pnpm v9 uses keys like '@scope/name@1.0.0' or 'name@1.0.0'
        if "@" not in pkg_key:
            continue  # should not happen for valid entries

        # Handle scoped packages: first @ is scope separator, last @ is version separators
        if pkg_key.startswith("@"):
            # '@scope/name@version'
            parts = pkg_key.split("@")
            # parts = ['', 'scope/name', 'version']
            scope_and_name = "@" + parts[1]  # '@scope/name'
            version = parts[2]
        else:
            # 'name@version'
            # split on last @ to handle package names containing @ (rare but possible)
            # For typical case, split on @
            parts = pkg_key.rsplit("@", 1)
            scope_and_name = parts[0]
            version = parts[1]

        # Avoid duplicates
        dedup_key = f"{scope_and_name}@{version}"
        if dedup_key in seen:
            continue
        seen.add(dedup_key)

        # Determine hash algorithm and convert to hex
        if integrity.startswith("sha512-"):
            sha_key = "sha512"
        elif integrity.startswith("sha256-"):
            sha_key = "sha256"
        elif integrity.startswith("sha1-"):
            sha_key = "sha1"
        else:
            print(f"WARNING: Unknown integrity format for {pkg_key}: {integrity[:30]}", file=sys.stderr)
            continue

        hex_hash = sri_to_hex(integrity)
        url = make_npm_url(scope_and_name, version)
        dest = make_dest(scope_and_name, version)

        entry = {
            "type": "file",
            "url": url,
            sha_key: hex_hash,
            "dest": dest,
        }
        entries.append(entry)

    # Sort for deterministic output
    entries.sort(key=lambda e: e["dest"])

    with open("flatpak/node-sources.json", "w") as f:
        json.dump(entries, f, indent=2)

    print(f"Generated flatpak/node-sources.json with {len(entries)} package entries")
    print(f"File size: {Path('flatpak/node-sources.json').stat().st_size} bytes")


if __name__ == "__main__":
    main()