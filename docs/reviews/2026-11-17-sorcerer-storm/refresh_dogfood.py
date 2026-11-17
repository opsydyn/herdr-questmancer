"""Refresh only a revalidated, previously test-owned UI; keep the party open."""
from pathlib import Path
import hashlib
import json
import sys
import time

trial = Path(sys.argv[1]).resolve()
review = Path(__file__).resolve().parent
sys.path.insert(0, str(trial))
from dogfood import cli, receipt, save, owned_ui, key, text, screen, expect

r = receipt()
assert r['retained'] and Path(r['trial_root']).resolve() == trial
pane = r['managed_pane']
if '--resume-after-exit' not in sys.argv:
    assert owned_ui() == pane
assert pane not in r['protected_panes']
snapshot = cli('api', 'snapshot')['snapshot']
live = {p['pane_id']: p for p in snapshot['panes']}
for item in r['panes']:
    p = live[item['pane_id']]
    assert p['terminal_id'] == item['terminal_id']
    assert p['agent'] == item['agent'] and p['agent_status'] == item['state']
    processes = cli('pane', 'process-info', '--pane', item['pane_id'])['process_info']['foreground_processes']
    assert any(str(trial/'receiver.py') in p.get('argv', []) for p in processes)
owned_path = trial/'state/owned-work.json'
owned_before = owned_path.read_bytes()
state_before = json.loads((trial/'state/state.json').read_text())
resume = '--resume-after-exit' in sys.argv
if resume:
    assert not (trial/'state/runtime.json').exists()
    old = {'pid': r['ui_pid']}
else:
    old = json.loads((trial/'state/runtime.json').read_text())
    processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
    assert any(p['pid'] == old['pid'] and p['name'] == 'questmancer' for p in processes)
    visible = screen()
    world = 'dogfood-mist · ? UNKNOWN' in visible and 'Chronicle observations withheld' in visible
    assert 'ADVENTURER' in visible or "LIBRARIAN'S LEDGER" in visible or world, 'Preserve unexpected UI state'
    (review/'live-before.txt').write_text(visible+'\n')
    key('esc', 'esc', 'esc')
    key('q')
    until = time.monotonic() + 5
    while (trial/'state/runtime.json').exists() and time.monotonic() < until:
        time.sleep(.1)
    assert not (trial/'state/runtime.json').exists(), 'UI did not shut down cleanly'
shell = cli('pane', 'get', pane)['pane']
assert shell['terminal_id'] == r['ui_terminal_id'] and not shell.get('agent')
processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
until = time.monotonic() + 3
while any(p['name'] not in ('zsh', 'bash', 'fish') for p in processes) and time.monotonic() < until:
    time.sleep(.1)
    processes = cli('pane', 'process-info', '--pane', pane)['process_info']['foreground_processes']
assert all(p['name'] in ('zsh', 'bash', 'fish') for p in processes)
cli('pane', 'send-text', pane, 'python3 '+str(trial/'launch.py'))
cli('pane', 'send-keys', pane, 'enter')
until = time.monotonic() + 6
while not (trial/'state/runtime.json').exists() and time.monotonic() < until:
    time.sleep(.1)
current = json.loads((trial/'state/runtime.json').read_text())
assert current['pane_id'] == pane and current['pid'] != old['pid']
owned_ui()
key('/')
text('dogfood-rest')
key('enter')
expect('Agent: dogfood-rest')
visible = expect('Sorcerer')
assert 'Resting' in visible
(review/'live-after.txt').write_text(visible+'\n')
assert owned_path.read_bytes() == owned_before
state_after = json.loads((trial/'state/state.json').read_text())
assert state_before['personas'] == state_after['personas']
snapshot = cli('api', 'snapshot')['snapshot']
live = {p['pane_id']: p for p in snapshot['panes']}
for item in r['panes']:
    p = live[item['pane_id']]
    assert p['terminal_id'] == item['terminal_id'] and p['agent'] == item['agent']
    assert p['agent_status'] == item['state']
assert not live[pane].get('agent')
assert all(p in live for p in r['protected_panes'])
candidate = hashlib.sha256((review.parents[2]/'target/release/questmancer').read_bytes()).hexdigest()
r.update(ui_pid=current['pid'], candidate_sha256=candidate, sorcerer_storm_refreshed=True)
save(r)
result = dict(pane_id=pane, previous_pid=old['pid'], new_pid=current['pid'],
              candidate_sha256=candidate, owned_store_unchanged=True, comparison_scope='After clean shutdown through relaunch' if resume else 'Full refresh',
              owned_matches_retained_revision_7=json.loads(owned_before) == json.loads((trial/'authority-before-restart.json').read_text()),
              saved_personas_unchanged=True, synthetic_party_retained=True,
              ui_not_an_adventurer=True, selected='dogfood-rest / Sorcerer / Resting',
              shared_server_restarted=False, native_appearance='Pending human review')
(review/'live-refresh.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result, indent=2))
