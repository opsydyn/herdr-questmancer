use std::{
    io::{ErrorKind, SeekFrom},
    path::Path,
};

use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::domain::{Chronicle, ChronicleEntry};

use super::{PersistenceDiagnostic, PersistenceError};

const MAX_INDIVIDUAL_DIAGNOSTICS: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayResult {
    pub chronicle: Chronicle,
    pub diagnostics: Vec<PersistenceDiagnostic>,
}

pub fn replay_chronicle(path: &Path, bytes: &[u8], maximum_entries: usize) -> ReplayResult {
    let mut chronicle = Chronicle::new(maximum_entries);
    let mut diagnostics = Vec::new();
    let mut rejected_records = 0_usize;
    let mut lines = bytes.split(|byte| *byte == b'\n').peekable();

    for line_number in 1.. {
        let Some(line) = lines.next() else {
            break;
        };
        let is_final_slice = lines.peek().is_none();
        if is_final_slice {
            if !line.is_empty() {
                record_diagnostic(
                    &mut diagnostics,
                    &mut rejected_records,
                    PersistenceDiagnostic {
                        operation: "replay chronicle",
                        path: path.to_owned(),
                        line: Some(line_number),
                        source_message: "truncated final chronicle record".to_owned(),
                    },
                );
            }
            break;
        }

        let text = match std::str::from_utf8(line) {
            Ok(text) => text,
            Err(error) => {
                record_diagnostic(
                    &mut diagnostics,
                    &mut rejected_records,
                    PersistenceDiagnostic {
                        operation: "decode chronicle record",
                        path: path.to_owned(),
                        line: Some(line_number),
                        source_message: error.to_string(),
                    },
                );
                continue;
            }
        };
        match serde_json::from_str::<ChronicleEntry>(text) {
            Ok(entry) => {
                chronicle.append(entry);
            }
            Err(error) => record_diagnostic(
                &mut diagnostics,
                &mut rejected_records,
                PersistenceDiagnostic {
                    operation: "parse chronicle record",
                    path: path.to_owned(),
                    line: Some(line_number),
                    source_message: error.to_string(),
                },
            ),
        }
    }

    if rejected_records > MAX_INDIVIDUAL_DIAGNOSTICS {
        diagnostics.push(PersistenceDiagnostic {
            operation: "replay chronicle",
            path: path.to_owned(),
            line: None,
            source_message: format!(
                "{} additional rejected chronicle records omitted",
                rejected_records - MAX_INDIVIDUAL_DIAGNOSTICS
            ),
        });
    }

    ReplayResult {
        chronicle,
        diagnostics,
    }
}

pub async fn load_chronicle(path: &Path, maximum_entries: usize) -> ReplayResult {
    match tokio::fs::read(path).await {
        Ok(bytes) => replay_chronicle(path, &bytes, maximum_entries),
        Err(error) if error.kind() == ErrorKind::NotFound => ReplayResult {
            chronicle: Chronicle::new(maximum_entries),
            diagnostics: Vec::new(),
        },
        Err(error) => ReplayResult {
            chronicle: Chronicle::new(maximum_entries),
            diagnostics: vec![PersistenceDiagnostic {
                operation: "read chronicle",
                path: path.to_owned(),
                line: None,
                source_message: error.to_string(),
            }],
        },
    }
}

pub async fn append_chronicle(path: &Path, entry: &ChronicleEntry) -> Result<(), PersistenceError> {
    let mut bytes = serde_json::to_vec(entry).map_err(|error| PersistenceError {
        operation: "serialize chronicle record",
        path: path.to_owned(),
        line: None,
        source_message: error.to_string(),
    })?;
    bytes.push(b'\n');

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| PersistenceError {
            operation: "create chronicle directory",
            path: parent.to_owned(),
            line: None,
            source_message: error.to_string(),
        })?;

    if tail_needs_separator(path).await? {
        bytes.insert(0, b'\n');
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|error| PersistenceError {
            operation: "open chronicle",
            path: path.to_owned(),
            line: None,
            source_message: error.to_string(),
        })?;
    file.write_all(&bytes)
        .await
        .map_err(|error| PersistenceError {
            operation: "write chronicle",
            path: path.to_owned(),
            line: None,
            source_message: error.to_string(),
        })?;
    file.sync_data().await.map_err(|error| PersistenceError {
        operation: "sync chronicle",
        path: path.to_owned(),
        line: None,
        source_message: error.to_string(),
    })
}

async fn tail_needs_separator(path: &Path) -> Result<bool, PersistenceError> {
    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(tail_inspection_error(path, &error)),
    };
    if metadata.len() == 0 || !metadata.is_file() {
        return Ok(false);
    }

    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|error| tail_inspection_error(path, &error))?;
    file.seek(SeekFrom::End(-1))
        .await
        .map_err(|error| tail_inspection_error(path, &error))?;
    let mut tail = [0_u8; 1];
    file.read_exact(&mut tail)
        .await
        .map_err(|error| tail_inspection_error(path, &error))?;
    Ok(tail[0] != b'\n')
}

fn tail_inspection_error(path: &Path, error: &std::io::Error) -> PersistenceError {
    PersistenceError {
        operation: "inspect chronicle tail",
        path: path.to_owned(),
        line: None,
        source_message: error.to_string(),
    }
}

fn record_diagnostic(
    diagnostics: &mut Vec<PersistenceDiagnostic>,
    rejected_records: &mut usize,
    diagnostic: PersistenceDiagnostic,
) {
    *rejected_records += 1;
    if *rejected_records <= MAX_INDIVIDUAL_DIAGNOSTICS {
        diagnostics.push(diagnostic);
    }
}
