"""Exercise assignment, restart, and downgrade protection in disposable offline PTYs."""

import argparse
import hashlib
import json
from pathlib import Path
import runpy
import tempfile


HERE = Path(__file__).resolve().parent
PREREQUISITES_DIR = HERE.parent / "2026-11-19-owned-quest-prerequisites"
PREVIOUS = runpy.run_path(str(PREREQUISITES_DIR / "pty-prerequisites.py"))
ROOT, SOURCE, QUEST, FIXTURE = (PREVIOUS[name] for name in ("ROOT", "SOURCE", "QUEST", "FIXTURE"))
TERMINAL, SELECT_QUEST, READ_JSON, ADD_PREREQUISITE, CACHE_HAS_LINK = (
    PREVIOUS[name] for name in ("TERMINAL", "SELECT_QUEST", "READ_JSON", "edit", "cache_has_link")
)


def cache_has_assignment(profile, revision, assignment):
    cache = profile / "work-state.json"
    if not cache.exists():
        return False
    owned = READ_JSON(cache)["projection"]["sources"].get(SOURCE)
    return (owned is not None and owned["cursor"] == revision
            and owned["inventory"]["quests"][QUEST]["assignment"] == assignment
            and not owned.get("bindings"))


def edit(screen, profile, authority, revision, name):
    original = authority.read_bytes()
    previous = json.loads(original)["state"]["inventory"]["quests"][QUEST]["assignment"]
    shown = previous["name"] if previous else "Unassigned"
    screen.see("y: assign")
    screen.send(b"y")
    screen.see(f"Current assignee: {shown}")
    screen.send(b"\t\t\r")
    screen.see("Choose Set assignee with a name")
    assert authority.read_bytes() == original
    screen.send(b"\x1b[Z\x1b[Z")
    screen.send(b"\x1b[B")
    if name is None:
        screen.send(b"\x1b[B")
        screen.see("Clear assignment")
        screen.send(b"\t")
    else:
        screen.see("Set assignee")
        screen.send(b"\t")
        screen.send(b"\x1b[200~" + name.encode() + b"\x1b[201~")
    screen.send(b"\t\r")
    screen.see("Saved; Board updated.")
    saved = READ_JSON(authority)
    assert saved["version"] == 5
    receipt = saved["state"]["receipts"][-1]
    assert receipt["revision"] == revision
    assert receipt["command"]["actor"] == "Questmancer"
    assert receipt["command"]["action"] == {
        "operation": "ChangeQuestAssignment", "id": QUEST,
        "previous": previous, "assignee": name,
    }
    expected = ({"worker": f"owned-assignee:{receipt['command']['id']}", "name": name}
                if name is not None else None)
    assert saved["state"]["inventory"]["quests"][QUEST]["assignment"] == expected
    screen.wait(lambda: cache_has_assignment(profile, revision, expected), "acknowledged assignment")
    screen.send(b"\x1b")
    screen.see("QUEST BOARD")
    screen.see(f"Recorded assignee: {name or 'Unassigned'}")
    screen.send(b"\r")
    screen.see("Observe unavailable")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, default=ROOT / "target/release/questmancer")
    parser.add_argument("--previous", type=Path, required=True, help="Prerequisite build from 230067c")
    parser.add_argument("--logs", type=Path, required=True)
    args = parser.parse_args()
    candidate = args.candidate.resolve(strict=True)
    previous = args.previous.resolve(strict=True)
    candidate_hash = hashlib.sha256(candidate.read_bytes()).hexdigest()
    previous_hash = hashlib.sha256(previous.read_bytes()).hexdigest()
    recorded_hash = READ_JSON(PREREQUISITES_DIR / "pty-receipt.json")["candidate_sha256"]
    assert candidate_hash != previous_hash
    args.logs.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="questmancer-assignment-", dir=Path("/tmp").resolve()) as name:
        profile = Path(name)
        authority = profile / "owned-work.json"
        authority.write_bytes(FIXTURE.read_bytes())
        (profile / "state.json").write_text(json.dumps({
            "schema_version": 2, "last_view": "guild", "experience": 0,
            "preferences": {"motion": "none", "character_set": "unicode", "color_mode": "xterm256"},
            "selected_persona": None, "personas": {}, "seen_attention": [],
            "owned_store": {"path": str(authority), "source": SOURCE},
        }) + "\n")
        with TERMINAL(previous, profile, args.logs / "seed-v4.log") as screen:
            screen.wait(lambda: CACHE_HAS_LINK(profile, 5, []), "original fixture opened")
            SELECT_QUEST(screen, "Planned")
            ADD_PREREQUISITE(screen, profile, authority, 6, "Add")
            screen.send(b"\x1b")
            screen.quit()
        v4 = authority.read_bytes()
        assert json.loads(v4)["version"] == 4

        with TERMINAL(candidate, profile, args.logs / "assign.log") as screen:
            screen.see("offline: launch from Herdr")
            SELECT_QUEST(screen, "Planned")
            assert authority.read_bytes() == v4
            edit(screen, profile, authority, 7, "Release reviewer")
            screen.send(b"\x1b")
            screen.quit()
        assigned = authority.read_bytes()

        with TERMINAL(candidate, profile, args.logs / "replace-and-clear.log") as screen:
            screen.see("offline: launch from Herdr")
            SELECT_QUEST(screen, "Planned")
            screen.see("Recorded assignee: Release reviewer")
            assert authority.read_bytes() == assigned
            screen.resize(60, 20)
            edit(screen, profile, authority, 8, "Mira Sánchez")
            edit(screen, profile, authority, 9, None)
            screen.send(b"\x1b")
            screen.quit()
        cleared = authority.read_bytes()
        original = json.loads(v4)["state"]
        final = json.loads(cleared)["state"]
        for field in ("obligations", "artifacts"):
            assert original["inventory"][field] == final["inventory"][field]
        assert original["reviews"] == final["reviews"]
        quest = final["inventory"]["quests"][QUEST]
        assert quest["dependencies"] == [f"{SOURCE}/web"]
        assert quest["state"] == "Planned"
        assert quest["assignment"] is None

        with TERMINAL(previous, profile, args.logs / "downgrade-refused.log") as screen:
            screen.see("offline: launch from Herdr")
            screen.send(b"b")
            screen.send(b"a")
            screen.see("unsupported version")
            assert authority.read_bytes() == cleared
            screen.send(b"\x1b")
            screen.send(b"\x1b")
            screen.quit()
        assert authority.read_bytes() == cleared
    assert not profile.exists()
    print(json.dumps({
        "baseline_commit": "230067c",
        "candidate_sha256": candidate_hash,
        "previous_prerequisite_sha256": previous_hash,
        "previous_matches_prerequisite_receipt": previous_hash == recorded_hash,
        "previous_seeded_v4_opened_without_rewrite": True,
        "acknowledged": [
            {"operation": "Assign", "revision": 7, "columns": 120, "rows": 34},
            {"operation": "Replace", "revision": 8, "columns": 60, "rows": 20},
            {"operation": "Clear", "revision": 9, "columns": 60, "rows": 20},
        ],
        "keep_current_preserved_bytes": True,
        "assignee_visible_after_process_restart": True,
        "observe_unavailable_without_verified_binding": True,
        "earlier_reader_refused_v5_after_clear": True,
        "prerequisites_lifecycle_obligations_reviews_and_artifacts_preserved": True,
        "terminal_teardown_and_test_profile_cleanup": True,
        "herdr_resources_created": 0,
    }, indent=2))


if __name__ == "__main__":
    main()
