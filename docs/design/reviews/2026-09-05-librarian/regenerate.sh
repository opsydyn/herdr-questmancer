#!/usr/bin/env bash
set -euo pipefail
review_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(git -C "$review_dir" rev-parse --show-toplevel)
review_python=${REVIEW_PYTHON:-python3}
review_tmp=$(mktemp -d "${TMPDIR:-/tmp}/questmancer-librarian-review.XXXXXX")
trap 'rm -rf -- "$review_tmp"' EXIT
cd -- "$repo_dir"
"$review_python" -c 'from PIL import Image, ImageDraw, ImageFont'
cargo build --lib --features storybook --message-format=json > "$review_tmp/build.jsonl"
"$review_python" - "$review_tmp" "$review_dir" "$repo_dir" <<'PY'
import json, subprocess, sys
from pathlib import Path
tmp, review, repo = map(Path, sys.argv[1:])
paths = {}
for line in (tmp/'build.jsonl').read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in {'questmancer', 'ratatui', 'serde_json'}:
        for filename in item['filenames']:
            if filename.endswith('.rlib'):
                paths[item['target']['name']] = filename
assert len(paths) == 3, 'Missing build artifacts for review export'
cmd = ['rustc', '--edition=2024', '-D', 'warnings', str(review/'export.rs'), '-L', 'dependency='+str(Path(paths['serde_json']).parent), '-o', str(tmp/'export')]
for name, path in paths.items():
    cmd += ['--extern', name+'='+path]
subprocess.run(cmd, check=True)
subprocess.run([str(tmp/'export'), str(review), str(tmp/'rendered.json')], check=True)
PY
"$review_python" "$review_dir/layout.py" "$review_tmp/rendered.json"
