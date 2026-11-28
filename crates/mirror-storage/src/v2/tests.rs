//! Failure injection around initialization's persistence/publication boundaries.
#![cfg(target_os = "linux")]

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "../../tests/support_v2/mod.rs"]
mod support;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "mirror-v2-init-fault-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fail() -> StorageErrorV2 {
    io::Error::other("injected initialization failure").into()
}

#[test]
fn prepublication_errors_never_expose_selected_state_or_cleanup_partial_files() {
    let chain = support::main_chain(4);
    for boundary in [
        InitBoundary::RootReady,
        InitBoundary::ManifestSynced,
        InitBoundary::BlockSynced(0),
        InitBoundary::BlockSynced(4),
        InitBoundary::BlocksDirectorySynced,
        InitBoundary::PreparedDirectorySynced,
        InitBoundary::SelectionStaged,
    ] {
        let scratch = Scratch::new();
        let root = scratch.0.join("store");
        let result = initialize_inner(&root, &chain, |at| {
            if at == boundary { Err(fail()) } else { Ok(()) }
        });
        assert!(
            matches!(result, Err(StorageErrorV2::InitializationIncomplete(_))),
            "{boundary:?}"
        );
        assert!(root.is_dir());
        assert!(!root.join(SELECTION_NAME).exists());
        let before = root_names(&root).unwrap();
        let reopened = StoreV2::inspect(&root, chain.manifest().identity(), chain.checkpoint());
        assert!(
            matches!(
                reopened,
                Err(StorageErrorV2::EmptyDirectory | StorageErrorV2::PartialInitialization)
            ),
            "{boundary:?}: {reopened:?}"
        );
        assert_eq!(root_names(&root).unwrap(), before);
        // No stale lock is left by an initialization error or failed inspection.
        let ownership = DirectoryLockV2::acquire(&root, StoreAccessV2::Writer).unwrap();
        drop(ownership);
        if boundary == InitBoundary::SelectionStaged {
            assert!(root.join(PENDING_NAME).is_file());
            assert!(matches!(
                StoreV2::initialize(&root, &chain),
                Err(StorageErrorV2::PartialInitialization)
            ));
            assert!(root.join(PENDING_NAME).is_file());
        }
    }
}

#[test]
fn postpublication_errors_return_no_handle_and_require_checked_reconciliation() {
    let chain = support::main_chain(4);
    for boundary in [
        InitBoundary::SelectionPublished,
        InitBoundary::SelectionDirectorySynced,
    ] {
        let scratch = Scratch::new();
        let root = scratch.0.join("store");
        let result = initialize_inner(&root, &chain, |at| {
            if at == boundary { Err(fail()) } else { Ok(()) }
        });
        assert!(matches!(
            result,
            Err(StorageErrorV2::PublicationUncertain(_))
        ));
        assert!(root.join(SELECTION_NAME).is_file());
        assert!(!root.join(PENDING_NAME).exists());
        let inspected =
            StoreV2::inspect(&root, chain.manifest().identity(), chain.checkpoint()).unwrap();
        assert_eq!(inspected.chain(), &chain);
        drop(inspected);
        let writer =
            StoreV2::open_writer(&root, chain.manifest().identity(), chain.checkpoint()).unwrap();
        assert_eq!(writer.chain(), &chain);
        drop(writer);
    }
}
