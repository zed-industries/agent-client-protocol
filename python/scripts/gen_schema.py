#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

def main() -> None:
    schema_json = ROOT / "schema" / "schema.json"
    out_py = ROOT / "python" / "src" / "acp" / "schema.py"
    if not schema_json.exists():
        print(f"Schema not found at {schema_json}. Run 'npm run generate:json-schema' first.", file=sys.stderr)
        sys.exit(1)
    cmd = [
        sys.executable,
        "-m",
        "datamodel_code_generator",
        "--input",
        str(schema_json),
        "--input-file-type",
        "jsonschema",
        "--output",
        str(out_py),
        "--target-python-version",
        "3.12",
        "--collapse-root-models",
        "--output-model-type",
        "pydantic_v2.BaseModel",
        "--use-annotated",
    ]
    subprocess.check_call(cmd)

if __name__ == "__main__":
    main()