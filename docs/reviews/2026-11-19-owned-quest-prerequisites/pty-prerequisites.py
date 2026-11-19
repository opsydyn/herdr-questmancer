"""Exercise prerequisite edits, restart, and downgrade protection in isolated PTYs."""

import argparse
import hashlib
import json
from pathlib import Path
import runpy
import tempfile


HERE = Path(__file__).resolve().parent
LIFECYCLE = runpy.run_path(str(HERE.parent / "2026-11-19-owned-quest-lifecycle/pty-lifecycle.py"))
ROOT, SOURCE, QUEST, FIXTURE = (LIFECYCLE[name] for name in ("ROOT", "SOURCE", "QUEST", "FIXTURE"))
TERMINAL, SELECT_QUEST, CHANGE_STATE, READ_JSON = (
    LIFECYCLE[name] for name in ("terminal", "select_quest", "change", "read_json")
)
PREREQUISITE = f"{SOURCE}/web"


def cache_has_link(profile, revision, dependencies):
    cache = profile / "work-state.json"
    if not cache.exists():
        return False
    owned = READ_JSON(cache)["projection"]["sources"].get(SOURCE)
    return (owned is not None and owned["cursor"] == revision
            and owned["inventory"]["quests"][QUEST]["dependencies"] == dependencies)


def edit(screen, profile, authority, revision, operation):
    previous = authority.read_bytes()
    screen.see("p: prerequisites")
    screen.send(b"p")
    screen.see("Dependent quest: Character picker")
    screen.send(b"\t\r")
    screen.see("Choose a prerequisite to add")
    assert authority.read_bytes() == previous
    screen.send(b"\x1b[Z")
    screen.send(b"\x1b[B")
    screen.see(f"{operation} Web migration")
    screen.send(b"\t\r")
    expected = [PREREQUISITE] if operation == "Add" else []
    screen.wait(lambda: cache_has_link(profile, revision, expected), "acknowledged link edit")
    screen.see("Saved; Board updated.")
    saved = READ_JSON(authority)
    assert saved["version"] == 4
    assert saved["state"]["inventory"]["quests"][QUEST]["dependencies"] == expected
    receipt = saved["state"]["receipts"][-1]
    assert receipt["revision"] == revision
    assert receipt["command"]["actor"] == "Questmancer"
    assert receipt["command"]["action"] == {
        "operation": "ChangeQuestPrerequisite", "id": QUEST,
        "prerequisite_id": PREREQUISITE, "change": operation,
    }
    screen.send(b"\x1b")
    screen.see("QUEST BOARD")


def refuse_cycle(screen, authority):
    previous = authority.read_bytes()
    screen.send(b"j")
    screen.see("Web migration | Commission | Planned")
    screen.send(b"p")
    screen.see("Dependent quest: Web migration")
    screen.send(b"\x1b[B")
    screen.see("Add Character picker")
    screen.send(b"\t\r")
    screen.see("This link would create a cycle")
    assert authority.read_bytes() == previous
    screen.send(b"\x1b")
    screen.send(b"\x1b")
    screen.see("QUEST BOARD")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, default=ROOT / "target/release/questmancer")
    parser.add_argument("--previous", type=Path, required=True, help="Lifecycle build from 0a9752f")
    parser.add_argument("--logs", type=Path, required=True)
    args = parser.parse_args()
    candidate = args.candidate.resolve(strict=True)
    previous = args.previous.resolve(strict=True)
    candidate_hash = hashlib.sha256(candidate.read_bytes()).hexdigest()
    previous_hash = hashlib.sha256(previous.read_bytes()).hexdigest()
    recorded_lifecycle_hash = READ_JSON(
        HERE.parent / "2026-11-19-owned-quest-lifecycle/pty-receipt.json"
    )["candidate_sha256"]
    assert candidate_hash != previous_hash
    args.logs.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="questmancer-prerequisites-", dir=Path("/tmp").resolve()) as name:
        profile = Path(name)
        authority = profile / "owned-work.json"
        authority.write_bytes(FIXTURE.read_bytes())
        (profile / "state.json").write_text(json.dumps({
            "schema_version": 2, "last_view": "guild", "experience": 0,
            "preferences": {"motion": "none", "character_set": "unicode", "color_mode": "xterm256"},
            "selected_persona": None, "personas": {}, "seen_attention": [],
            "owned_store": {"path": str(authority), "source": SOURCE},
        }) + "\n")
        with TERMINAL(previous, profile, args.logs / "seed-v3.log") as screen:
            screen.wait(lambda: cache_has_link(profile, 5, []), "original fixture opened")
            SELECT_QUEST(screen, "Planned")
            CHANGE_STATE(screen, profile, authority, 6, "Planned", "Active", "Mark active")
            screen.send(b"\x1b")
            screen.quit()
        v3 = authority.read_bytes()
        assert json.loads(v3)["version"] == 3

        with TERMINAL(candidate, profile, args.logs / "add.log") as screen:
            screen.see("offline: launch from Herdr")
            SELECT_QUEST(screen, "Active")
            assert authority.read_bytes() == v3
            edit(screen, profile, authority, 7, "Add")
            screen.see("Dependency: Web migration | Planned")
            refuse_cycle(screen, authority)
            screen.send(b"\x1b")
            screen.quit()
        added = authority.read_bytes()

        with TERMINAL(candidate, profile, args.logs / "remove-after-restart.log") as screen:
            screen.see("offline: launch from Herdr")
            SELECT_QUEST(screen, "Active")
            screen.see("Dependency: Web migration | Planned")
            assert authority.read_bytes() == added
            screen.resize(60, 20)
            edit(screen, profile, authority, 8, "Remove")
            screen.send(b"\x1b")
            screen.quit()
        removed = authority.read_bytes()
        original_state = json.loads(v3)["state"]
        final_state = json.loads(removed)["state"]
        for field in ("obligations", "artifacts"):
            assert final_state["inventory"][field] == original_state["inventory"][field]
        assert final_state["reviews"] == original_state["reviews"]
        assert final_state["inventory"]["quests"][QUEST]["state"] == "Active"

        with TERMINAL(previous, profile, args.logs / "downgrade-refused.log") as screen:
            screen.see("offline: launch from Herdr")
            screen.send(b"b")
            screen.send(b"a")
            screen.see("unsupported version")
            assert authority.read_bytes() == removed
            screen.send(b"\x1b")
            screen.send(b"\x1b")
            screen.quit()
        assert authority.read_bytes() == removed
    assert not profile.exists()
    print(json.dumps({
        "baseline_commit": "0a9752f",
        "candidate_sha256": candidate_hash,
        "previous_lifecycle_sha256": previous_hash,
        "previous_matches_lifecycle_receipt": previous_hash == recorded_lifecycle_hash,
        "previous_seeded_v3_opened_without_rewrite": True,
        "acknowledged": [
            {"operation": "Add", "revision": 7, "columns": 120, "rows": 34},
            {"operation": "Remove", "revision": 8, "columns": 60, "rows": 20},
        ],
        "unselected_saves_and_cycle_preserved_bytes": True,
        "link_visible_after_process_restart": True,
        "earlier_reader_refused_v4_after_last_link_removed": True,
        "lifecycle_obligations_review_and_artifacts_preserved": True,
        "terminal_teardown_and_test_profile_cleanup": True,
        "herdr_resources_created": 0,
    }, indent=2))


if __name__ == "__main__":
    main()
