//! Bounded local control-directory I/O and cooperative Linux inode ownership.
//! The operator and parent directories are trusted. These checks detect static
//! unsafe paths and accidental replacement, not hostile same-user namespace races.

use std::{
    ffi::OsString,
    fs::{self, File, Metadata, OpenOptions},
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::{
    fs::TryLockError,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
};

use super::{MAX_CONTROL_READ_BYTES_V2, NodeControlErrorV2};

pub(super) struct ControlLock {
    directory: File,
}

impl ControlLock {
    pub(super) fn acquire(root: &Path, write: bool) -> Result<Self, NodeControlErrorV2> {
        require_platform()?;
        #[cfg(target_os = "linux")]
        {
            let before = directory_metadata(root)?;
            let directory = File::open(root)?;
            let opened = directory.metadata()?;
            same_directory(&before, &opened)?;
            let result = if write {
                directory.try_lock()
            } else {
                directory.try_lock_shared()
            };
            match result {
                Ok(()) => {}
                Err(TryLockError::WouldBlock) => return Err(NodeControlErrorV2::Busy),
                Err(TryLockError::Error(error)) => return Err(error.into()),
            }
            same_directory(&directory_metadata(root)?, &opened)?;
            Ok(Self { directory })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, write);
            Err(NodeControlErrorV2::UnsupportedPlatform)
        }
    }

    pub(super) fn verify(&self, root: &Path) -> Result<(), NodeControlErrorV2> {
        same_directory(&directory_metadata(root)?, &self.directory.metadata()?)
    }

    pub(super) fn sync(&self, root: &Path) -> Result<(), NodeControlErrorV2> {
        self.verify(root)?;
        self.directory.sync_all()?;
        self.verify(root)
    }
}

pub(super) fn require_platform() -> Result<(), NodeControlErrorV2> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(NodeControlErrorV2::UnsupportedPlatform)
    }
}

/// Resolve only lexical current-directory components. Never follow a symlink,
/// accept `..`, create parents, or accept the filesystem root as managed data.
fn checked_root(input: &Path) -> Result<PathBuf, NodeControlErrorV2> {
    if input.as_os_str().is_empty() {
        return Err(NodeControlErrorV2::InvalidPath);
    }
    let absolute = if input.is_absolute() {
        input.to_path_buf()
    } else {
        std::env::current_dir()?.join(input)
    };
    let mut result = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir | Component::Normal(_) => result.push(component),
            Component::CurDir => {}
            _ => return Err(NodeControlErrorV2::InvalidPath),
        }
    }
    if result.parent().is_none() {
        return Err(NodeControlErrorV2::InvalidPath);
    }
    let mut prefix = PathBuf::new();
    for component in result.components() {
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(NodeControlErrorV2::UnsafePath(
                        "non-directory or symbolic-link path component",
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound && prefix == result => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(result)
}

pub(super) fn checked_pair(
    control: &Path,
    store: &Path,
) -> Result<(PathBuf, PathBuf), NodeControlErrorV2> {
    require_platform()?;
    let control = checked_root(control)?;
    let store = checked_root(store)?;
    if control.starts_with(&store) || store.starts_with(&control) {
        return Err(NodeControlErrorV2::UnsafePath(
            "control and store roots must be separate and non-nested",
        ));
    }
    #[cfg(target_os = "linux")]
    if let (Ok(left), Ok(right)) = (fs::symlink_metadata(&control), fs::symlink_metadata(&store)) {
        if left.dev() == right.dev() && left.ino() == right.ino() {
            return Err(NodeControlErrorV2::UnsafePath(
                "control and store alias one directory inode",
            ));
        }
    }
    Ok((control, store))
}

pub(super) fn directory_exists(root: &Path) -> Result<bool, NodeControlErrorV2> {
    match fs::symlink_metadata(root) {
        Ok(_) => {
            directory_metadata(root)?;
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn create_directory(root: &Path) -> Result<(), NodeControlErrorV2> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(target_os = "linux")]
    builder.mode(0o700);
    builder.create(root)?;
    Ok(())
}

pub(super) fn sync_parent(root: &Path) -> Result<(), NodeControlErrorV2> {
    let parent = root.parent().ok_or(NodeControlErrorV2::InvalidPath)?;
    directory_metadata(parent)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn directory_metadata(root: &Path) -> Result<Metadata, NodeControlErrorV2> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(NodeControlErrorV2::UnsafePath(
            "expected a non-symlink directory",
        ));
    }
    Ok(metadata)
}

fn same_directory(before: &Metadata, after: &Metadata) -> Result<(), NodeControlErrorV2> {
    if !before.is_dir() || !after.is_dir() {
        return Err(NodeControlErrorV2::UnsafePath("directory identity changed"));
    }
    #[cfg(target_os = "linux")]
    if before.dev() != after.dev() || before.ino() != after.ino() {
        return Err(NodeControlErrorV2::UnsafePath(
            "locked directory inode was replaced",
        ));
    }
    Ok(())
}

pub(super) fn names(root: &Path) -> Result<Vec<OsString>, NodeControlErrorV2> {
    // The fifth name is rejected immediately; no unbounded enumeration/allocation.
    let mut names = Vec::with_capacity(4);
    for entry in fs::read_dir(root)? {
        if names.len() == 4 {
            return Err(NodeControlErrorV2::InvalidLayout(
                "too many control-directory entries",
            ));
        }
        names.push(entry?.file_name());
    }
    names.sort();
    Ok(names)
}

pub(super) fn regular_metadata(path: &Path) -> Result<Metadata, NodeControlErrorV2> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(NodeControlErrorV2::UnsafePath(
            "managed entry is not a regular non-symlink file",
        ));
    }
    #[cfg(target_os = "linux")]
    if metadata.nlink() != 1 {
        return Err(NodeControlErrorV2::UnsafePath("hardlinked managed file"));
    }
    Ok(metadata)
}

fn same_regular(before: &Metadata, opened: &Metadata) -> Result<(), NodeControlErrorV2> {
    if !opened.is_file() || before.len() != opened.len() {
        return Err(NodeControlErrorV2::LengthMismatch);
    }
    #[cfg(target_os = "linux")]
    if before.dev() != opened.dev() || before.ino() != opened.ino() || opened.nlink() != 1 {
        return Err(NodeControlErrorV2::UnsafePath(
            "managed file identity changed",
        ));
    }
    Ok(())
}

pub(super) struct ReadBudget {
    remaining: u64,
}

impl ReadBudget {
    pub(super) const fn new() -> Self {
        Self {
            remaining: MAX_CONTROL_READ_BYTES_V2,
        }
    }

    fn charge(&mut self, bytes: u64) -> Result<(), NodeControlErrorV2> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(NodeControlErrorV2::ReadBudgetExceeded)?;
        Ok(())
    }
}

fn read_open(
    path: &Path,
    maximum: u64,
    exact: Option<u64>,
    budget: &mut ReadBudget,
) -> Result<(Vec<u8>, File), NodeControlErrorV2> {
    let before = regular_metadata(path)?;
    if before.len() > maximum || exact.is_some_and(|length| length != before.len()) {
        return Err(NodeControlErrorV2::LengthMismatch);
    }
    let amount = before
        .len()
        .checked_add(1)
        .ok_or(NodeControlErrorV2::LengthMismatch)?;
    budget.charge(amount)?;
    let mut file = File::open(path)?;
    same_regular(&before, &file.metadata()?)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(
            usize::try_from(amount).map_err(|_| NodeControlErrorV2::AllocationFailed)?,
        )
        .map_err(|_| NodeControlErrorV2::AllocationFailed)?;
    (&mut file).take(amount).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != before.len() {
        return Err(NodeControlErrorV2::LengthMismatch);
    }
    same_regular(&before, &file.metadata()?)?;
    same_regular(&before, &regular_metadata(path)?)?;
    Ok((bytes, file))
}

pub(super) fn read(
    path: &Path,
    maximum: u64,
    exact: Option<u64>,
    budget: &mut ReadBudget,
) -> Result<Vec<u8>, NodeControlErrorV2> {
    Ok(read_open(path, maximum, exact, budget)?.0)
}

pub(super) fn verify_exact(
    path: &Path,
    expected: &[u8],
    budget: &mut ReadBudget,
) -> Result<(), NodeControlErrorV2> {
    let (actual, _) = read_open(
        path,
        expected.len() as u64,
        Some(expected.len() as u64),
        budget,
    )?;
    if actual != expected {
        return Err(NodeControlErrorV2::ChangedControl);
    }
    Ok(())
}

pub(super) fn sync_exact(
    path: &Path,
    expected: &[u8],
    budget: &mut ReadBudget,
) -> Result<(), NodeControlErrorV2> {
    let (actual, file) = read_open(
        path,
        expected.len() as u64,
        Some(expected.len() as u64),
        budget,
    )?;
    if actual != expected {
        return Err(NodeControlErrorV2::ChangedControl);
    }
    file.sync_all()?;
    same_regular(&file.metadata()?, &regular_metadata(path)?)?;
    Ok(())
}

pub(super) fn create_file(path: &Path) -> Result<File, NodeControlErrorV2> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(target_os = "linux")]
    options.mode(0o600);
    let file = options.open(path)?;
    same_regular(&regular_metadata(path)?, &file.metadata()?)?;
    Ok(file)
}

pub(super) fn append_file(
    path: &Path,
    expected: &[u8],
    budget: &mut ReadBudget,
) -> Result<File, NodeControlErrorV2> {
    verify_exact(path, expected, budget)?;
    let before = regular_metadata(path)?;
    let file = OpenOptions::new().append(true).open(path)?;
    same_regular(&before, &file.metadata()?)?;
    Ok(file)
}
