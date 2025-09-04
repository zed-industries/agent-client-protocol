#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path.cwd()

def main() -> None:
    meta_json = ROOT / "schema" / "meta.json"
    out_py = ROOT / "python" / "src" / "acp" / "meta.py"
    data = json.loads(meta_json.read_text())
    agent_methods = data.get("agentMethods", {})
    client_methods = data.get("clientMethods", {})
    version = data.get("version", 1)
    out_py.write_text(
        "# This file is generated from schema/meta.json. Do not edit by hand.\n"
        f"AGENT_METHODS = {repr(agent_methods)}\n"
        f"CLIENT_METHODS = {repr(client_methods)}\n"
        f"PROTOCOL_VERSION = {int(version)}\n"
    )

if __name__ == "__main__":
    main()
