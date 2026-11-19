//! Bounded synthetic filesystem demonstration, never a production initializer.
use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_storage::v2::{StorageErrorV2, StoreAccessV2, StoreV2};

// Published synthetic actual-block vectors; there are no real user/device inputs.
#[path = "../tests/support_v2/mod.rs"]
mod fixture;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        // Only a parent created successfully by this invocation is owned here.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parent = std::env::temp_dir().join(format!(
        "mirror-storage-v2-demo-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
    ));
    fs::create_dir(&parent)?;
    let scratch = Scratch(parent);
    let root = scratch.0.join("selected-main");
    let chain = fixture::main_chain(4);
    let identity = fixture::fixture_identity();
    let expected = fixture::main_expectation(4);

    println!("Mirror V2 — explicit synthetic storage");
    println!("Initializing five independently vectored accepted blocks...");
    let store = StoreV2::initialize(&root, &chain)?;
    assert_eq!(store.access(), StoreAccessV2::Writer);
    assert!(matches!(
        StoreV2::open_writer(&root, identity, expected),
        Err(StorageErrorV2::Busy)
    ));
    println!("Second writer: rejected (ownership held)");
    drop(store);

    let inspected = StoreV2::inspect(&root, identity, expected)?;
    assert_eq!(inspected.access(), StoreAccessV2::ReadOnly);
    assert_eq!(inspected.chain(), &chain);
    println!("Reopened height: {}", inspected.chain().height());
    println!("Reopened tip: {}", inspected.chain().tip_hash());
    println!(
        "Replayed ledger root: {}",
        inspected.chain().checkpoint().ledger_root
    );
    println!("Full accounts, membership and budgets: reproduced");
    drop(inspected);

    // A different membership history stays a separately selected store. This
    // demonstration neither elects a canonical branch nor performs a reorg.
    let fork = fixture::fork_chain();
    let fork_root = scratch.0.join("selected-fork");
    drop(StoreV2::initialize(&fork_root, &fork)?);
    let recovered_fork = StoreV2::inspect(&fork_root, identity, fixture::fork_expectation())?;
    assert_eq!(recovered_fork.chain(), &fork);
    assert!(StoreV2::inspect(&fork_root, identity, fixture::main_expectation(3)).is_err());
    println!("Different membership branch: independently replayed; wrong branch pin rejected");
    drop(recovered_fork);
    println!("Storage initialization and checked reopening: VALID");
    println!("Generation updates, durable reorgs and live V2 networking remain later work.");
    Ok(())
}
