#!/usr/bin/env bash
set -euo pipefail
review_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(git -C "$review_dir" rev-parse --show-toplevel)
review_python=${REVIEW_PYTHON:-python3}
review_tmp=$(mktemp -d "${TMPDIR:-/tmp}/questmancer-pty-review.XXXXXX")
trap 'rm -rf -- "$review_tmp"' EXIT
cd -- "$repo_dir"
cargo build --lib --bin questmancer-storybook --features storybook --message-format=json > "$review_tmp/build.jsonl"
"$review_python" - "$review_tmp" "$review_dir" <<'PY'
import json,subprocess,sys
from pathlib import Path
tmp,review=map(Path,sys.argv[1:]);paths={};binary=None
for line in (tmp/'build.jsonl').read_text().splitlines():
    item=json.loads(line)
    if item.get('reason')!='compiler-artifact':continue
    if item['target']['name']=='questmancer-storybook':binary=item.get('executable')
    if item['target']['name'] in {'questmancer','serde_json'}:
        for filename in item['filenames']:
            if filename.endswith('.rlib'):paths[item['target']['name']]=filename
assert len(paths)==2 and binary
cmd=['rustc','--edition=2024','-D','warnings',str(review/'catalogue.rs'),'-L','dependency='+str(Path(paths['serde_json']).parent),'-o',str(tmp/'catalogue')]
for name,path in paths.items():cmd+=['--extern',name+'='+path]
subprocess.run(cmd,check=True)
(tmp/'catalogue.json').write_bytes(subprocess.check_output([str(tmp/'catalogue')]))
subprocess.run([sys.executable,str(review/'pty_check.py'),binary,str(tmp/'catalogue.json'),str(review/'result.json')],check=True)
PY
