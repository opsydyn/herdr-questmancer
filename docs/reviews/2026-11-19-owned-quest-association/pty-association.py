"""Qualify the release UI against a disposable Herdr-protocol socket."""

import argparse
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import runpy
import shlex
import socketserver
import subprocess
import sys
import tempfile
import time

HERE = Path(__file__).resolve().parent
LIFECYCLE = runpy.run_path(str(HERE.parent / "2026-11-19-owned-quest-lifecycle/pty-lifecycle.py"))
ROOT, SOURCE, FIXTURE, TERMINAL, SELECT_QUEST = (
    LIFECYCLE[name] for name in ("ROOT", "SOURCE", "FIXTURE", "terminal", "select_quest")
)
SESSION = "synthetic-association-session"
PANE = "w1:p1"
FOCUS_CALLS = {"pane.get", "pane.focus"}


def serve(profile):
    snapshot = json.loads((ROOT / "tests/fixtures/herdr/session_snapshot.json").read_bytes())[
        "result"
    ]
    agent = snapshot["snapshot"]["agents"][0]
    agent["agent_session"] = {
        "source": "test", "agent": "codex", "kind": "session", "value": SESSION,
    }
    pong = json.loads((ROOT / "tests/fixtures/herdr/pong.json").read_bytes())["result"]

    class Handler(socketserver.StreamRequestHandler):
        def handle(self):
            request = json.loads(self.rfile.readline())
            method = request["method"]
            with (profile / "methods.log").open("a") as calls:
                calls.write(method + "\n")
            reply = {"id": request["id"]}
            if method == "ping":
                reply["result"] = pong
            elif method == "session.snapshot":
                reply["result"] = snapshot
            elif method == "events.subscribe":
                reply["result"] = {"type": "subscription_started"}
            elif method in FOCUS_CALLS:
                assert request["params"]["pane_id"] == PANE
                pane = {**agent, "focused": method == "pane.focus"}
                if (profile / "replace-terminal").exists():
                    pane["terminal_id"] = "replacement-terminal"
                reply["result"] = {"type": "pane_info", "pane": pane}
            else:
                reply["error"] = {"code": "method_not_found", "message": "outside this probe"}
            self.wfile.write(json.dumps(reply).encode() + b"\n")
            self.wfile.flush()
            if method == "events.subscribe":
                self.rfile.read()

    with socketserver.ThreadingUnixStreamServer(str(profile / "herdr.sock"), Handler) as server:
        server.serve_forever()


@contextmanager
def protocol_fixture(profile):
    process = subprocess.Popen([sys.executable, __file__, "--serve", str(profile)])
    try:
        deadline = time.monotonic() + 5
        while not (profile / "herdr.sock").exists():
            if time.monotonic() >= deadline or process.poll() is not None:
                raise RuntimeError("socket fixture did not start")
            time.sleep(0.03)
        yield
    finally:
        if process.poll() is None:
            process.terminate()
        process.wait(timeout=5)


def focus_methods(profile):
    return [method for method in (profile / "methods.log").read_text().splitlines()
            if method in FOCUS_CALLS]


def run(candidate, logs):
    logs.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="qm-association-", dir=Path("/tmp").resolve()) as name:
        profile = Path(name)
        authority = profile / "owned-work.json"
        original = FIXTURE.read_bytes()
        authority.write_bytes(original)
        (profile / "state.json").write_text(json.dumps({
            "schema_version": 2, "last_view": "guild", "experience": 0,
            "preferences": {"motion": "none", "character_set": "unicode", "color_mode": "xterm256"},
            "selected_persona": None, "personas": {}, "seen_attention": [],
            "owned_store": {"path": str(authority), "source": SOURCE},
        }) + "\n")
        launcher = profile / "launch"
        launcher.write_text(
            "#!/bin/sh\n"
            f"export HERDR_SOCKET_PATH={shlex.quote(str(profile / 'herdr.sock'))}\n"
            "export HERDR_BIN_PATH=/usr/bin/false HERDR_PANE_ID=w1:managed\n"
            f"exec {shlex.quote(str(candidate))} \"$@\"\n"
        )
        launcher.chmod(0o700)
        with protocol_fixture(profile):
            with TERMINAL(launcher, profile, logs / "association.log") as screen:
                screen.see("adventurer")
                SELECT_QUEST(screen, "Planned")
                screen.see("Selected adventurer to link")
                screen.send(b"\r")
                screen.see("Observe unavailable")
                assert focus_methods(profile) == []
                screen.send(b"l")
                screen.see("Live association:")
                assert focus_methods(profile) == []
                screen.send(b"\r")
                screen.see(f"observing {PANE}")
                assert focus_methods(profile) == ["pane.get", "pane.focus"]
                (profile / "replace-terminal").touch()
                screen.send(b"\r")
                screen.see("The adventurer session has changed")
                assert focus_methods(profile) == ["pane.get", "pane.focus", "pane.get"]
                assert "Live association:" not in screen.screen_text()
                screen.send(b"\x1b")
                screen.quit()
            (profile / "replace-terminal").unlink()
            with TERMINAL(launcher, profile, logs / "restart-unlinked.log") as screen:
                screen.see("adventurer")
                SELECT_QUEST(screen, "Planned")
                screen.see("Selected adventurer to link")
                screen.send(b"\r")
                screen.see("Observe unavailable")
                assert focus_methods(profile) == ["pane.get", "pane.focus", "pane.get"]
                screen.send(b"\x1b")
                screen.quit()
        assert authority.read_bytes() == original
        for filename in ("state.json", "work-state.json", "chronicle.jsonl"):
            saved = profile / filename
            if saved.exists():
                assert SESSION.encode() not in saved.read_bytes()
    assert not profile.exists()
    print(json.dumps({
        "baseline_commit": "e01485d",
        "candidate_sha256": hashlib.sha256(candidate.read_bytes()).hexdigest(),
        "socket": "disposable Herdr protocol 22 fixture",
        "linked_observe": ["pane.get", "pane.focus"],
        "stale_observe": ["pane.get"],
        "stale_link_cleared_on_verification_failure": True,
        "unlinked_observe_and_restart_observe": [],
        "owned_store_bytes_preserved": True,
        "raw_session_absent_from_persisted_profile": True,
        "terminal_teardown_and_test_profile_cleanup": True,
        "live_herdr_resources_created": 0,
    }, indent=2))


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--serve":
        serve(Path(sys.argv[2]))
    else:
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument("--candidate", type=Path, default=ROOT / "target/release/questmancer")
        parser.add_argument("--logs", type=Path, required=True)
        args = parser.parse_args()
        run(args.candidate.resolve(strict=True), args.logs)
