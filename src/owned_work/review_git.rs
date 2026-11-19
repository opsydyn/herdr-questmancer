//! Bounded, read-only Git evidence for a selected immutable revision pair.
use super::ReviewTarget;
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const OUTPUT_LIMIT: u64 = 64 * 1024;
const DEADLINE: Duration = Duration::from_secs(3);

pub(crate) fn changes(target: &ReviewTarget) -> Result<Vec<String>, String> {
    if !target.valid() {
        return Err("Use an absolute repository path and two distinct full commit IDs".into());
    }
    let root = git(&target.repository, &["rev-parse", "--show-toplevel"])?;
    let root = std::str::from_utf8(&root)
        .map_err(|_| "Git repository root is not UTF-8")?
        .trim_end_matches('\n');
    if Path::new(root) != target.repository
        || std::fs::canonicalize(&target.repository).ok().as_ref() != Some(&target.repository)
    {
        return Err("Select the physical root of the local Git checkout".into());
    }
    for revision in [&target.base, &target.head] {
        let commit = format!("{revision}^{{commit}}");
        let resolved = git(
            &target.repository,
            &["rev-parse", "--verify", "--end-of-options", &commit],
        )?;
        if resolved.strip_suffix(b"\n") != Some(revision.as_bytes()) {
            return Err("Both pinned revisions must identify local commits".into());
        }
    }
    let output = git(
        &target.repository,
        &[
            "diff-tree",
            "--no-commit-id",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--name-status",
            "-z",
            "-r",
            &target.base,
            &target.head,
            "--",
        ],
    )?;
    if output.is_empty() {
        return Ok(Vec::new());
    }
    let mut fields = output
        .strip_suffix(&[0])
        .ok_or("Incomplete Git result")?
        .split(|b| *b == 0);
    let mut files = Vec::new();
    while let Some(status) = fields.next() {
        let path = fields.next().ok_or("Incomplete Git result")?;
        // Escape control characters before names reach the terminal. Git's -z retains them.
        let mut name = String::new();
        for c in String::from_utf8_lossy(path).chars() {
            if c.is_control() {
                name.extend(c.escape_default());
            } else {
                name.push(c);
            }
        }
        files.push(format!("{} {name}", String::from_utf8_lossy(status)));
    }
    Ok(files)
}

fn git(path: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("GIT_") {
            command.env_remove(name);
        }
    }
    let mut child = command
        .current_dir(path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "--no-optional-locks",
            "--no-replace-objects",
            "-c",
            "protocol.allow=never",
            "-c",
            "core.fsmonitor=false",
        ])
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not start local Git: {error}"))?;
    let stdout = child.stdout.take().expect("Git stdout is piped");
    std::thread::scope(|scope| {
        let output = scope.spawn(move || {
            let mut bytes = Vec::new();
            stdout.take(OUTPUT_LIMIT + 1).read_to_end(&mut bytes)?;
            Ok::<_, std::io::Error>(bytes)
        });
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if started.elapsed() < DEADLINE => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                result => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(result
                        .err()
                        .map_or_else(|| "Local Git timed out".into(), |e| e.to_string()));
                }
            }
        };
        let bytes = output
            .join()
            .map_err(|_| "Git output reader failed")?
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > OUTPUT_LIMIT {
            return Err("Changed-file summary exceeds the 64 KiB review limit".into());
        }
        if !status?.success() {
            return Err("Git could not read the selected local repository or commits".into());
        }
        Ok(bytes)
    })
}
