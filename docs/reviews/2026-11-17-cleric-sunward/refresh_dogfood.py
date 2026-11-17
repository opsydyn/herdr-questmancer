"""Revalidate and refresh the retained test-owned UI; preserve every adventurer."""
from pathlib import Path
import hashlib
import json
import sys
import time

trial = Path(sys.argv[1]).resolve()
review = Path(__file__).resolve().parent
sys.path.insert(0, str(trial))
from dogfood import cli, receipt, save, owned_ui, key, screen

r = receipt()
assert r['retained'] and Path(r['trial_root']).resolve() == trial
pane = owned_ui()
assert pane not in r['protected_panes']
snapshot = cli('api', 'snapshot')['snapshot']
live = {p['pane_id']: p for p in snapshot['panes']}
assert live[pane]['terminal_id'] == r['ui_terminal_id'] and not live[pane].get('agent')
synthetic_before = {}
for item in r['panes']:
    p = live[item['pane_id']]
    assert p['terminal_id'] == item['terminal_id'] and p['agent'] == item['agent']
    processes = cli('pane', 'process-info', '--pane', item['pane_id'])['process_info']['foreground_processes']
    assert any(str(trial/'receiver.py') in p.get('argv', []) for p in processes)
    synthetic_before[item['pane_id']] = p['agent_status']
old = json.loads((trial/'state/runtime.json').read_text())
processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
assert any(p['pid'] == old['pid'] and p['name'] == 'questmancer' for p in processes)
visible = screen()
world = 'NEEDS COUNSEL' in visible and 'Chronicle observations withheld' in visible
assert 'ADVENTURER' in visible or "LIBRARIAN'S LEDGER" in visible or world, 'Preserve unexpected UI state'
(review/'live-before.txt').write_text(visible+'\n')
owned_path = trial/'state/owned-work.json'
owned_before = owned_path.read_bytes()
state_before = json.loads((trial/'state/state.json').read_text())
persona_digest = lambda state: hashlib.sha256(json.dumps(state['personas'], sort_keys=True).encode()).hexdigest()
baseline = dict(pane_id=pane, terminal_id=r['ui_terminal_id'], old_pid=old['pid'],
                owned_sha256=hashlib.sha256(owned_before).hexdigest(),
                personas_sha256=persona_digest(state_before), synthetic_states=synthetic_before)
(review/'live-baseline.json').write_text(json.dumps(baseline, indent=2)+'\n')

key('esc', 'esc', 'esc')
key('q')
until = time.monotonic() + 5
while (trial/'state/runtime.json').exists() and time.monotonic() < until:
    time.sleep(.1)
assert not (trial/'state/runtime.json').exists(), 'UI did not shut down cleanly'
until = time.monotonic() + 3
while True:
    p = cli('pane', 'get', pane)['pane']
    assert p['terminal_id'] == r['ui_terminal_id'] and not p.get('agent')
    processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
    if processes and all(p['name'] in ('zsh', 'bash', 'fish') for p in processes):
        break
    assert time.monotonic() < until, 'Foreground did not return to the owned shell'
    time.sleep(.1)

cli('pane', 'send-text', pane, 'python3 '+str(trial/'launch.py'))
cli('pane', 'send-keys', pane, 'enter')
until = time.monotonic() + 6
while not (trial/'state/runtime.json').exists() and time.monotonic() < until:
    time.sleep(.1)
current = json.loads((trial/'state/runtime.json').read_text())
assert current['pane_id'] == pane and current['pid'] != old['pid']
owned_ui()
key('esc', '2', 'esc')
until = time.monotonic() + 5
while True:
    visible = screen()
    if 'NEEDS COUNSEL' in visible and 'Chronicle observations withheld' in visible:
        break
    assert time.monotonic() < until, 'Party did not reappear'
    time.sleep(.1)
(review/'live-after.txt').write_text(visible+'\n')
assert owned_path.read_bytes() == owned_before
state_after = json.loads((trial/'state/state.json').read_text())
assert persona_digest(state_after) == baseline['personas_sha256']
snapshot = cli('api', 'snapshot')['snapshot']
live = {p['pane_id']: p for p in snapshot['panes']}
for item in r['panes']:
    p = live[item['pane_id']]
    assert p['terminal_id'] == item['terminal_id'] and p['agent'] == item['agent']
    assert p['agent_status'] == synthetic_before[item['pane_id']]
assert not live[pane].get('agent')
assert all(p in live for p in r['protected_panes'])
processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
assert any(p['pid'] == current['pid'] and p['name'] == 'questmancer' for p in processes)
candidate = hashlib.sha256((review.parents[2]/'target/release/questmancer').read_bytes()).hexdigest()
r.update(ui_pid=current['pid'], candidate_sha256=candidate, cleric_sunward_refreshed=True)
save(r)
result = dict(pane_id=pane, previous_pid=old['pid'], new_pid=current['pid'],
              candidate_sha256=candidate, owned_store_unchanged=True,
              owned_sha256=baseline['owned_sha256'], saved_personas_unchanged=True,
              synthetic_party_retained=True, ui_not_an_adventurer=True,
              left_view='Delve / card closed; no agent selected by the harness',
              shared_server_restarted=False, native_appearance='Pending human review')
(review/'live-refresh.json').write_text(json.dumps(result, indent=2)+'\n')
logs = cli('plugin', 'log', 'list', '--plugin', 'opsydyn.questmancer', '--limit', '50')
(review/'plugin-logs-final.json').write_text(json.dumps(logs, indent=2)+'\n')
print(json.dumps(result, indent=2))
