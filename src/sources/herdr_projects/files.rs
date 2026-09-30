//! Descriptor-relative reads: NOFOLLOW at every component, no create/write flag.
use super::contract::ReadError;
use rustix::{
    fd::OwnedFd,
    fs::{self, Dir, Mode, OFlags},
};
use std::{
    fs::File,
    io::Read,
    path::{Component, Path},
};

const READ: OFlags = OFlags::RDONLY
    .union(OFlags::CLOEXEC)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK);
pub(super) const MAX_FILE: usize = 256 * 1024;
#[derive(Debug)]
pub(super) struct Directory {
    fd: OwnedFd,
}
impl Directory {
    pub(super) fn root(path: &Path) -> Result<Self, ReadError> {
        let mut fd = fs::open("/", READ | OFlags::DIRECTORY, Mode::empty())
            .map_err(|e| io("project", e.to_string()))?;
        for part in path.components() {
            match part {
                Component::RootDir => {}
                Component::Normal(name) => {
                    fd = fs::openat(&fd, name, READ | OFlags::DIRECTORY, Mode::empty())
                        .map_err(|e| io("project", e.to_string()))?;
                }
                _ => return Err(ReadError::Selection),
            }
        }
        Ok(Self { fd })
    }
    pub(super) fn child(&self, name: &str) -> Result<Self, ReadError> {
        let fd = fs::openat(&self.fd, name, READ | OFlags::DIRECTORY, Mode::empty())
            .map_err(|e| io(name, e.to_string()))?;
        Ok(Self { fd })
    }
    pub(super) fn file(&self, name: &str) -> Result<Option<File>, ReadError> {
        let fd = match fs::openat(&self.fd, name, READ, Mode::empty()) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(e) => return Err(io(name, e.to_string())),
        };
        let file = File::from(fd);
        let meta = file.metadata().map_err(|e| io(name, e.to_string()))?;
        if !meta.is_file() {
            return Err(io(name, "expected a regular file"));
        }
        if meta.len() > MAX_FILE as u64 {
            return Err(ReadError::Limit);
        }
        Ok(Some(file))
    }
    pub(super) fn bytes(&self, name: &str) -> Result<Option<Vec<u8>>, ReadError> {
        let Some(file) = self.file(name)? else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        file.take((MAX_FILE + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| io(name, e.to_string()))?;
        if bytes.len() > MAX_FILE {
            return Err(ReadError::Limit);
        }
        Ok(Some(bytes))
    }
    pub(super) fn names(&self) -> Result<Vec<String>, ReadError> {
        let entries = Dir::read_from(&self.fd).map_err(|e| io("threads", e.to_string()))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| io("threads", e.to_string()))?;
            let name = entry
                .file_name()
                .to_str()
                .map_err(|e| io("threads", e.to_string()))?;
            if name != "." && name != ".." {
                names.push(name.to_owned());
            }
            if names.len() > 4096 {
                return Err(ReadError::Limit);
            }
        }
        names.sort();
        Ok(names)
    }
    pub(super) fn identity(&self) -> Result<(i128, u64), ReadError> {
        identity(&self.fd)
    }
}
pub(super) fn identity(fd: impl std::os::fd::AsFd) -> Result<(i128, u64), ReadError> {
    let stat = fs::fstat(fd).map_err(|e| io("source identity", e.to_string()))?;
    Ok((i128::from(stat.st_dev), stat.st_ino))
}
pub(super) fn io(path: &str, message: impl Into<String>) -> ReadError {
    ReadError::Io {
        path: path.into(),
        message: message.into(),
    }
}
