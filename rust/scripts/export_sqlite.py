#!/usr/bin/env python3
"""Export the OpenAlbion SQLite database into JSON data files for the Rust API.

Usage:
    python3 export_sqlite.py [sqlite_path] [output_dir]

Defaults:
    sqlite_path = ../database/database.sqlite (relative to this script's dir)
    output_dir  = ../data

Each table is written as a bare JSON array of row objects, preserving the
database's natural row order (matching Laravel's `->get()` ordering).
Columns containing JSON text are parsed into real nested JSON.
Timestamps (created_at / updated_at) are not exported — the API never
exposes them.
"""

import json
import sqlite3
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent

# table -> columns that store JSON text and must be parsed
TABLES = {
    "categories": [],
    "weapons": [],
    "armors": [],
    "accessories": [],
    "consumables": [],
    "spells": ["attributes"],
    "weapon_spells": [],
    "armor_spells": [],
    "accessory_spells": [],
    "weapon_stats": ["stats"],
    "armor_stats": ["stats"],
    "accessory_stats": ["stats"],
    "consumable_stats": ["stats"],
    "consumable_craftings": ["requirements"],
}

# columns to skip entirely
EXCLUDED_COLUMNS = {"created_at", "updated_at"}


def export(sqlite_path: Path, output_dir: Path) -> None:
    con = sqlite3.connect(sqlite_path)
    con.row_factory = sqlite3.Row
    output_dir.mkdir(parents=True, exist_ok=True)

    for table, json_columns in TABLES.items():
        rows = con.execute(f"SELECT * FROM {table}").fetchall()
        out_rows = []
        bad_json = 0
        for row in rows:
            obj = {}
            for key in row.keys():
                if key in EXCLUDED_COLUMNS:
                    continue
                value = row[key]
                if key in json_columns and value is not None:
                    try:
                        value = json.loads(value)
                    except (TypeError, ValueError):
                        bad_json += 1
                obj[key] = value
            out_rows.append(obj)

        out_path = output_dir / f"{table}.json"
        out_path.write_text(json.dumps(out_rows, ensure_ascii=False), encoding="utf-8")
        print(f"{table}: {len(out_rows)} rows -> {out_path.name}"
              + (f" ({bad_json} rows kept raw: unparseable JSON column)" if bad_json else ""))

    con.close()


def main() -> None:
    sqlite_path = Path(sys.argv[1]) if len(sys.argv) > 1 else SCRIPT_DIR.parent.parent / "database" / "database.sqlite"
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else SCRIPT_DIR.parent / "data"

    if not sqlite_path.exists():
        sys.exit(f"SQLite database not found: {sqlite_path}")

    print(f"Exporting {sqlite_path} -> {output_dir}")
    export(sqlite_path, output_dir)
    print("Done.")


if __name__ == "__main__":
    main()
