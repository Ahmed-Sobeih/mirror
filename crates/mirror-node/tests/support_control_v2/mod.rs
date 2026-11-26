//! Public synthetic histories and privately owned directories for node evidence.
#![allow(dead_code, unused_imports)]

#[path = "../../../mirror-network/tests/support_peer_process_v2/mod.rs"]
mod existing;

pub use existing::{
    Scratch, assert_complete_state, checkpoint, history, preferred_fork, store_bytes,
};

use mirror_chain::v2::ChainV2;
use mirror_network::v2::{HistoryFramesV2, HistoryReceiverV2, ReceivedCandidateV2};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn control_root(scratch: &Scratch) -> PathBuf {
    scratch.0.join("control")
}

pub fn received(chain: &ChainV2) -> ReceivedCandidateV2 {
    HistoryFramesV2::new(chain)
        .unwrap()
        .try_fold(
            HistoryReceiverV2::new(history::independent_identity()),
            |receiver, frame| receiver.accept_frame(&frame.unwrap()),
        )
        .unwrap()
        .finish()
        .unwrap()
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub kind: &'static str,
    pub device: u64,
    pub inode: u64,
    pub links: u64,
    pub modified: (i64, i64),
    pub bytes: Vec<u8>,
    pub symlink: Option<PathBuf>,
}

/// Application no-write checks deliberately exclude operating-system atime.
/// Symlinks are recorded without following them; these paths belong to fixtures.
#[cfg(target_os = "linux")]
pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, SnapshotEntry> {
    use std::os::unix::fs::MetadataExt;
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, SnapshotEntry>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let kind = metadata.file_type();
        entries.insert(
            path.strip_prefix(root).unwrap().to_path_buf(),
            SnapshotEntry {
                kind: if kind.is_dir() {
                    "directory"
                } else if kind.is_symlink() {
                    "symlink"
                } else if kind.is_file() {
                    "file"
                } else {
                    "special"
                },
                device: metadata.dev(),
                inode: metadata.ino(),
                links: metadata.nlink(),
                modified: (metadata.mtime(), metadata.mtime_nsec()),
                bytes: if kind.is_file() {
                    fs::read(path).unwrap()
                } else {
                    Vec::new()
                },
                symlink: if kind.is_symlink() {
                    Some(fs::read_link(path).unwrap())
                } else {
                    None
                },
            },
        );
        if kind.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}
