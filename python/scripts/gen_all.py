#!/usr/bin/env python3
from __future__ import annotations

import runpy
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent


def main() -> None:
    runpy.run_path(str(SCRIPTS / "gen_schema.py"), run_name="__main__")
    runpy.run_path(str(SCRIPTS / "gen_meta.py"), run_name="__main__")


if __name__ == "__main__":
    main()
