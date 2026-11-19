"""Verify the release build through an isolated PTY with a pre-lifecycle store."""

from contextlib import contextmanager
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import pty
import re
import select
import signal
import struct
import subprocess
import tempfile
import termios
import time

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests/fixtures/owned-work/pre-lifecycle-v2.json"
SOURCE = "owned:promo"
QUEST = f"{SOURCE}/picker"
CSI = re.compile(r"\x1b\[([0-?]*)[ -/]*([@-~])")
REPLIES = {
    b"\x1b_Gi=31": b"\x1b_Gi=31;ENOTSUP\x1b\\",
    b"\x1b[16t": b"\x1b[6;16;8t",
    b"\x1b[6n": b"\x1b[1;1R",
    b"\x1b[c": b"\x1b[?62;22c",
    b"\x1b[5n": b"\x1b[0n",
}


class Terminal:
    def __init__(self, binary, profile, log):
        self.master, self.slave = pty.openpty()
        self.output = bytearray()
        self.replied = set()
        self.log = log
        self.resize(120, 34)
        env = {name: value for name, value in os.environ.items()
               if not name.startswith(("HERDR_", "TERM_PROGRAM", "LC_TERMINAL", "KITTY_", "ITERM", "TMUX"))}
        env.update(HERDR_PLUGIN_STATE_DIR=str(profile), TERM="xterm-256color")
        self.process = subprocess.Popen(
            [binary, "ui"], stdin=self.slave, stdout=self.slave, stderr=self.slave,
            env=env, preexec_fn=self.claim_terminal,
        )

    def claim_terminal(self):
        os.setsid()
        fcntl.ioctl(self.slave, termios.TIOCSCTTY, 0)

    def resize(self, width, height):
        self.width, self.height = width, height
        self.resize_mark = len(self.output)
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ,
                    struct.pack("HHHH", height, width, width * 8, height * 16))
        if hasattr(self, "process"):
            self.process.send_signal(signal.SIGWINCH)

    def pump(self, seconds=0.15):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            if not select.select([self.master], [], [], max(0, deadline - time.monotonic()))[0]:
                break
            self.output.extend(os.read(self.master, 65536))
            for query, response in REPLIES.items():
                if query not in self.replied and query in self.output:
                    os.write(self.master, response)
                    self.replied.add(query)

    def send(self, keys):
        os.write(self.master, keys)
        self.pump()

    def wait(self, condition, label):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            self.pump(0.05)
            if condition():
                return
        raise AssertionError(label)

    def screen_text(self):
        """Apply Ratatui cursor-addressed updates to the current PTY grid."""
        data = self.output[self.resize_mark:].decode(errors="replace")
        grid = [[" "] * self.width for _ in range(self.height)]
        col = row = offset = 0
        while offset < len(data):
            char = data[offset]
            if char == "\x1b":
                csi = CSI.match(data, offset)
                if csi:
                    raw, command = csi.groups()
                    args = [int(part) if part.isdecimal() else 0 for part in raw.split(";")]
                    count = args[0] or 1
                    if command in ("H", "f"):
                        row = count - 1
                        col = ((args[1] if len(args) > 1 else 1) or 1) - 1
                    elif command == "G":
                        col = count - 1
                    elif command == "d":
                        row = count - 1
                    elif command == "J" and args[0] == 2:
                        grid = [[" "] * self.width for _ in range(self.height)]
                    elif command == "K" and 0 <= row < self.height:
                        grid[row][col:] = [" "] * len(grid[row][col:])
                    offset = csi.end()
                    continue
                if data[offset + 1:offset + 2] in ("_", "]", "P"):
                    end = data.find("\x1b\\", offset + 2)
                    offset = len(data) if end < 0 else end + 2
                    continue
                offset += 2
                continue
            if char == "\r":
                col = 0
            elif char == "\n":
                row += 1
            elif ord(char) >= 32:
                if 0 <= row < self.height and 0 <= col < self.width:
                    grid[row][col] = char
                col += 1
            offset += 1
        return "\n".join("".join(row) for row in grid)

    def see(self, label):
        self.wait(lambda: label in self.screen_text(), label)

    def quit(self):
        self.send(b"q")
        self.wait(lambda: self.process.poll() is not None, "normal terminal exit")
        assert self.process.returncode == 0
        for sequence in (b"\x1b[?1049l", b"\x1b[?2004l", b"\x1b[?25h"):
            assert sequence in self.output[-256:], f"missing terminal teardown {sequence!r}"

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait(timeout=3)
        self.log.write_bytes(self.output)
        os.close(self.master)
        os.close(self.slave)


@contextmanager
def terminal(binary, profile, log):
    screen = Terminal(binary, profile, log)
    try:
        yield screen
    finally:
        screen.close()


def read_json(path):
    return json.loads(path.read_bytes())


def cache_state(profile, revision, state):
    cache = profile / "work-state.json"
    if not cache.exists():
        return False
    owned = read_json(cache)["projection"]["sources"].get(SOURCE)
    return (owned is not None and owned["cursor"] == revision
            and owned["inventory"]["quests"][QUEST]["state"] == state)


def select_quest(screen, state):
    screen.send(b"b")
    screen.see("QUEST BOARD")
    screen.send(b"j")
    screen.send(b"j")
    screen.see(f"Character picker | Commission | {state}")


def change(screen, profile, authority, revision, before, after, label):
    original = authority.read_bytes()
    screen.send(b"t")
    screen.see(f"{label} · {before} → {after}")
    screen.send(b"\t\r")
    screen.see("Choose Confirm this change")
    assert authority.read_bytes() == original
    screen.send(b"\x1b[Z")
    screen.send(b"\x1b[B")
    screen.send(b"\t\r")
    screen.wait(lambda: cache_state(profile, revision, after), f"acknowledged {after}")
    saved = read_json(authority)
    assert saved["version"] == 3
    assert saved["state"]["inventory"]["quests"][QUEST]["state"] == after
    assert saved["state"]["receipts"][-1]["revision"] == revision
    assert saved["state"]["receipts"][-1]["command"]["actor"] == "Questmancer"
    screen.see("Saved; Board updated.")
    screen.send(b"\x1b")
    screen.see("QUEST BOARD")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, default=ROOT / "target/release/questmancer")
    parser.add_argument("--previous", type=Path, required=True, help="Published v0.2.4 executable")
    parser.add_argument("--logs", type=Path, required=True, help="New or existing evidence directory")
    args = parser.parse_args()
    candidate, previous = (item.resolve(strict=True) for item in (args.candidate, args.previous))
    assert subprocess.check_output([previous, "--version"], text=True).strip() == "questmancer 0.2.4"
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    assert sha(candidate) != sha(previous)
    args.logs.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="questmancer-lifecycle-pty-", dir=Path("/tmp").resolve()) as name:
        profile = Path(name)
        authority = profile / "owned-work.json"
        old = FIXTURE.read_bytes()
        authority.write_bytes(old)
        (profile / "state.json").write_text(json.dumps({
            "schema_version": 2, "last_view": "guild", "experience": 0,
            "preferences": {"motion": "none", "character_set": "unicode", "color_mode": "xterm256"},
            "selected_persona": None, "personas": {}, "seen_attention": [],
            "owned_store": {"path": str(authority), "source": SOURCE},
        }) + "\n")
        with terminal(candidate, profile, args.logs / "lifecycle-pty.log") as screen:
            screen.wait(lambda: cache_state(profile, 5, "Planned"), "legacy projection")
            assert authority.read_bytes() == old
            select_quest(screen, "Planned")
            change(screen, profile, authority, 6, "Planned", "Active", "Mark active")
            screen.resize(60, 20)
            screen.pump(0.25)
            change(screen, profile, authority, 7, "Active", "Completed", "Mark complete")
            screen.send(b"\x1b")
            screen.quit()
        completed = authority.read_bytes()
        with terminal(candidate, profile, args.logs / "lifecycle-restart-pty.log") as screen:
            screen.see("offline: launch from Herdr")
            select_quest(screen, "Completed")
            assert authority.read_bytes() == completed
            change(screen, profile, authority, 8, "Completed", "Active", "Reopen quest")
            screen.send(b"\x1b")
            screen.quit()
        active = authority.read_bytes()
        left = json.loads(old)["state"]
        right = json.loads(active)["state"]
        assert right["inventory"]["obligations"] == left["inventory"]["obligations"]
        assert right["reviews"] == left["reviews"]
        with terminal(previous, profile, args.logs / "lifecycle-previous-pty.log") as screen:
            screen.see("offline: launch from Herdr")
            screen.send(b"b")
            screen.send(b"a")
            screen.see("unsupported version")
            assert authority.read_bytes() == active
            screen.send(b"\x1b")
            screen.send(b"\x1b")
            screen.quit()
        assert authority.read_bytes() == active
    assert not profile.exists()
    result = {
        "candidate_sha256": sha(candidate),
        "published_0_2_4_sha256": sha(previous),
        "acknowledged": [
            {"revision": 6, "state": "Active", "columns": 120, "rows": 34},
            {"revision": 7, "state": "Completed", "columns": 60, "rows": 20},
            {"revision": 8, "state": "Active", "columns": 120, "rows": 34},
        ],
        "legacy_open_kept_bytes": True,
        "confirmed_after_restart": True,
        "open_summons_and_review_unchanged": True,
        "published_binary_refused_v3_and_preserved_bytes": True,
        "normal_quit_and_terminal_teardown_sequences": True,
        "test_directory_removed": True,
        "herdr_resources_created": 0,
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
