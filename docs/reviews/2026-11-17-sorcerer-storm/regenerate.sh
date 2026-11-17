#!/usr/bin/env bash
set -euo pipefail
review_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(git -C "$review_dir" rev-parse --show-toplevel)
review_python=${REVIEW_PYTHON:-python3}
review_tmp=$(mktemp -d "${TMPDIR:-/tmp}/questmancer-sorcerer-production.XXXXXX")
trap 'rm -rf -- "$review_tmp"' EXIT
cd -- "$repo_dir"
"$review_python" -c 'from PIL import Image, ImageDraw, ImageFont'
CARGO_INCREMENTAL=0 cargo build --lib --features storybook --message-format=json > "$review_tmp/build.jsonl"
"$review_python" - "$review_tmp" "$review_dir" <<'PY'
import json, subprocess, sys
from pathlib import Path
temporary, review = map(Path, sys.argv[1:])
libraries = {}
for line in (temporary/'build.jsonl').read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in {'questmancer','ratatui','serde_json'}:
        for filename in item['filenames']:
            if filename.endswith('.rlib'): libraries[item['target']['name']] = filename
assert len(libraries) == 3, 'Missing review build artifacts'
for source, output in [('export.rs', None), ('golden.rs', 'canonical-delve.json')]:
    command = ['rustc','--edition=2024','-D','warnings',str(review/source),'-L','dependency='+str(Path(libraries['serde_json']).parent),'-o',str(temporary/'export')]
    for name, filename in libraries.items(): command += ['--extern',name+'='+filename]
    subprocess.run(command,check=True)
    subprocess.run([str(temporary/'export'),str(review/output) if output else str(review)],check=True)
PY
"$review_python" "$review_dir/layout.py"
