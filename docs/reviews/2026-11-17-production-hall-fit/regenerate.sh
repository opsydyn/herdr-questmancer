#!/usr/bin/env bash
set -euo pipefail
review_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(git -C "$review_dir" rev-parse --show-toplevel)
review_tmp=$(mktemp -d "${TMPDIR:-/tmp}/questmancer-production-fit-export.XXXXXX")
trap 'rm -rf -- "$review_tmp"' EXIT
cd -- "$repo_dir"
cargo build --release --lib --message-format=json > "$review_tmp/build.jsonl"
python3 - "$review_tmp" "$review_dir" <<'PY'
import json, subprocess, sys
from pathlib import Path
temporary,review=map(Path,sys.argv[1:]);libraries={}
for line in (temporary/'build.jsonl').read_text().splitlines():
    item=json.loads(line)
    if item.get('reason')=='compiler-artifact' and item['target']['name'] in {'questmancer','ratatui'}:
        for filename in item['filenames']:
            if filename.endswith('.rlib'):libraries[item['target']['name']]=filename
assert len(libraries)==2
command=['rustc','--edition=2024','-D','warnings',str(review/'export.rs'),'-L','dependency='+str(Path(libraries['ratatui']).parent),'-o',str(temporary/'export')]
for name,filename in libraries.items():command+=['--extern',name+'='+filename]
subprocess.run(command,check=True);subprocess.run([str(temporary/'export'),str(review)],check=True)
PY
