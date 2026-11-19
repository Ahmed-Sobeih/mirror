//! Cooperative Linux directory-inode ownership for one store handle.
//!
//! The root directory must never be renamed, replaced or removed while a store
//! handle exists. Its parent tree is trusted. Static path checks do not claim
//! protection against hostile concurrent same-user namespace changes.

use std::{fs::File, path::Path};

#[cfg(target_os = "linux")]
use std::{
    fs::{self, Metadata, TryLockError},
    io,
    os::unix::fs::MetadataExt,
};

use super::{StorageErrorV2, StoreAccessV2};

/// Owns an uncloned directory descriptor until the containing store is dropped.
/// No lock file, PID lease, explicit unlock, or path deletion is involved.
#[derive(Debug)]
pub(super) struct DirectoryLockV2 {
    directory: File,
}

impl DirectoryLockV2 {
    pub(super) fn acquire(root: &Path, access: StoreAccessV2) -> Result<Self, StorageErrorV2> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, access);
            Err(StorageErrorV2::UnsupportedPlatform)
        }

        #[cfg(target_os = "linux")]
        {
            let before = checked_path_metadata(root)?;
            let directory = File::open(root).map_err(path_io_error)?;
            let opened = directory.metadata().map_err(StorageErrorV2::Io)?;
            if !opened.is_dir() {
                return Err(StorageErrorV2::NotDirectory);
            }
            if before.dev() != opened.dev() || before.ino() != opened.ino() {
                return Err(StorageErrorV2::UnsafePath(
                    "store directory changed while opening",
                ));
            }

            let result = match access {
                StoreAccessV2::Writer => directory.try_lock(),
                StoreAccessV2::ReadOnly => directory.try_lock_shared(),
            };
            match result {
                Ok(()) => {}
                Err(TryLockError::WouldBlock) => return Err(StorageErrorV2::Busy),
                Err(TryLockError::Error(error)) => return Err(StorageErrorV2::Io(error)),
            }

            // Detect a static replacement across the lock operation too. The
            // trusted-parent contract still excludes later malicious renames.
            let after = checked_path_metadata(root)?;
            if after.dev() != opened.dev() || after.ino() != opened.ino() {
                return Err(StorageErrorV2::UnsafePath(
                    "store directory changed while locking",
                ));
            }

            Ok(Self { directory })
        }
    }

    pub(super) fn sync(&self) -> Result<(), StorageErrorV2> {
        self.directory.sync_all().map_err(StorageErrorV2::Io)
    }
}

#[cfg(target_os = "linux")]
fn checked_path_metadata(root: &Path) -> Result<Metadata, StorageErrorV2> {
    let metadata = fs::symlink_metadata(root).map_err(path_io_error)?;
    if metadata.file_type().is_symlink() {
        return Err(StorageErrorV2::UnsafePath(
            "store directory is a symbolic link",
        ));
    }
    if !metadata.is_dir() {
        return Err(StorageErrorV2::NotDirectory);
    }
    Ok(metadata)
}

#[cfg(target_os = "linux")]
fn path_io_error(error: io::Error) -> StorageErrorV2 {
    if error.kind() == io::ErrorKind::NotFound {
        StorageErrorV2::MissingDirectory
    } else {
        StorageErrorV2::Io(error)
    }
}
