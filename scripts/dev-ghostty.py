#!/usr/bin/env python3
"""Open the source-linked plugin in an isolated, unpatched Herdr session."""

import argparse
import fcntl
import json
import os
from pathlib import Path
import shlex
import shutil
import struct
import subprocess
import sys
import tempfile
import termios
import time


SCRIPT = Path(__file__).resolve()
ROOT = SCRIPT.parent.parent
PLUGIN = "opsydyn.questmancer"


def wait_for_geometry():
    deadline = time.monotonic() + 5
    while True:
        try:
            size = fcntl.ioctl(sys.stdin.fileno(), termios.TIOCGWINSZ, bytes(8))
        except OSError:
            return
        rows, columns, width, height = struct.unpack("HHHH", size)
        if all((rows, columns, width, height)) and width % columns == height % rows == 0:
            return
        if time.monotonic() >= deadline:
            print("Pixel geometry unavailable. Questmancer will use its terminal fallback.", flush=True)
            return
        time.sleep(0.05)


def environment(run):
    env = {key: value for key, value in os.environ.items()
           if not key.startswith("HERDR_") and key not in ("BASH_ENV", "ENV", "NO_COLOR")}
    env.update(
        XDG_CONFIG_HOME=str(run / "config"),
        XDG_STATE_HOME=str(run / "state"),
        XDG_CACHE_HOME=str(run / "cache"),
        HERDR_CONFIG_PATH=str(run / "config.toml"),
        HERDR_SOCKET_PATH=str(run / "herdr.sock"),
    )
    return env


def herdr_command(herdr, env, *args):
    return subprocess.run([str(herdr), *args], env=env, cwd=ROOT, check=True,
                          capture_output=True, text=True, timeout=15).stdout


def wait_for_open(herdr, env):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        result = json.loads(herdr_command(herdr, env, "plugin", "log", "list", "--plugin", PLUGIN))
        for action in result["result"]["logs"]:
            if action["action_id"] == "open" and action["status"] != "running":
                if action["status"] == "succeeded":
                    return
                raise SystemExit(action["stderr"].strip() or "Questmancer's open action failed.")
        time.sleep(0.05)
    raise SystemExit("Timed out waiting for Questmancer's open action.")


def verify_server(herdr, env, socket_inode):
    socket = Path(env["HERDR_SOCKET_PATH"])
    if not socket.is_socket() or socket.stat().st_ino != socket_inode:
        raise SystemExit(f"Cannot find the original dev server at {socket}.")
    status = json.loads(herdr_command(herdr, env, "status", "server", "--json"))
    if not status["running"] or Path(status["socket"]).resolve() != socket:
        raise SystemExit(f"Herdr did not confirm the isolated socket at {socket}.")


def attach(herdr, run):
    if run.parent != Path("/tmp").resolve() or not run.name.startswith("qm-ghostty-"):
        raise SystemExit("Use the isolated session directory printed by the dev launcher.")
    record = json.loads((run / "dev-session.json").read_text())
    if Path(record["herdr"]) != herdr:
        raise SystemExit("Reattach with the same Herdr executable used to boot the dev session.")
    env = environment(run)
    verify_server(herdr, env, record["socket_inode"])
    return subprocess.run([str(herdr)], env=env, cwd=ROOT).returncode


def boot(herdr):
    subprocess.run(["cargo", "build", "--locked", "--release"], cwd=ROOT, check=True)

    run = Path(tempfile.mkdtemp(prefix="qm-ghostty-", dir="/tmp")).resolve()
    env = environment(run)
    Path(env["HERDR_CONFIG_PATH"]).write_text(
        'onboarding = false\n[terminal]\nkitty_graphics = true\n'
        'default_shell = "/bin/sh"\nshell_mode = "non_login"\n'
        '[session]\nresume_agents_on_restore = false\n'
        '[update]\nversion_check = false\nmanifest_check = false\n'
    )
    gate = run / "geometry.bash"
    wait_command = shlex.join([sys.executable, str(SCRIPT), "--wait-for-geometry"])
    gate.write_text(
        'unset BASH_ENV\n'
        f'if [[ ${{HERDR_PLUGIN_ID:-}} == {PLUGIN} && '
        '${HERDR_PLUGIN_ENTRYPOINT_ID:-} == guild-hall ]]; then\n'
        f'    {wait_command}\n'
        'fi\n'
    )
    log = run / "server.log"
    with log.open("wb") as output:
        server = subprocess.Popen([str(herdr), "server"], cwd=run,
                                  env=dict(env, BASH_ENV=str(gate)),
                                  stdin=subprocess.DEVNULL, stdout=output,
                                  stderr=subprocess.STDOUT, start_new_session=True)
    try:
        socket = Path(env["HERDR_SOCKET_PATH"])
        deadline = time.monotonic() + 5
        while not socket.is_socket():
            if server.poll() is not None or time.monotonic() >= deadline:
                raise SystemExit(f"The isolated Herdr server did not start. See {log}.")
            time.sleep(0.05)
        inode = socket.stat().st_ino
        verify_server(herdr, env, inode)
        herdr_command(herdr, env, "workspace", "create", "--cwd", str(ROOT),
                      "--label", "Questmancer Ghostty dev", "--focus")
        herdr_command(herdr, env, "plugin", "link", str(ROOT))
        herdr_command(herdr, env, "plugin", "action", "invoke", f"{PLUGIN}.open")
        wait_for_open(herdr, env)
        session = {"herdr": str(herdr), "socket_inode": inode}
        (run / "dev-session.json").write_text(json.dumps(session) + "\n")
    except BaseException:
        if server.poll() is None:
            server.terminate()
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait()
        raise

    reattach = shlex.join([sys.executable, str(SCRIPT), "--attach", str(run)])
    print(f"Isolated Questmancer dev session at {run}", flush=True)
    print(f"Reattach with {reattach}", flush=True)
    return attach(herdr, run)


def main():
    if sys.argv[1:] == ["--wait-for-geometry"]:
        wait_for_geometry()
        return 0
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--attach", type=Path, help="reattach to an existing dev session directory")
    args = parser.parse_args()
    if (os.environ.get("HERDR_ENV") or os.environ.get("HERDR_PANE_ID")
            or os.environ.get("TERM_PROGRAM", "").casefold() == "herdr"):
        parser.exit(1, "Open a new Ghostty window and run this from its plain shell.\n")
    if not sys.stdin.isatty() or not sys.stdout.isatty():
        parser.exit(1, "The Ghostty dev launcher needs an interactive terminal.\n")
    executable = shutil.which("herdr")
    if not executable:
        parser.exit(1, "Install Herdr before running the Ghostty dev boot.\n")
    herdr = Path(executable).resolve()
    try:
        return attach(herdr, args.attach.resolve()) if args.attach else boot(herdr)
    except subprocess.CalledProcessError as error:
        parser.exit(1, error.stderr or f"{error}\n")
    except (OSError, subprocess.TimeoutExpired, ValueError, KeyError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    sys.exit(main())
