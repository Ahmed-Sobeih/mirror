//! Bounded accepted-block publication in newly created synthetic storage only.
use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_storage::v2::{AppendStorageErrorV2, AppendStoreV2, StorageErrorV2};

#[path = "../tests/support_v2/mod.rs"]
mod fixture;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!(
        "mirror-append-v2-demo-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
    ));
    fs::create_dir(&path)?;
    let scratch = Scratch(path);
    let root = scratch.0.join("selected-chain");
    let identity = fixture::fixture_identity();
    let history = fixture::main_chain(4);
    let mut store = AppendStoreV2::initialize(&root, &fixture::main_chain(0))?;

    println!("Mirror V2 — durable accepted-block append");
    println!("Initialized explicit append profile at height 0 / generation 0");
    assert!(matches!(
        AppendStoreV2::open_writer(&root, identity, store.checkpoint()?),
        Err(AppendStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
    println!("Second writer: rejected while directory ownership is held");

    let original = store.checkpoint()?;
    assert!(store.append(history.blocks()[2].clone()).is_err());
    assert_eq!(store.checkpoint()?, original);
    assert!(!store.is_fenced());
    println!("Wrong-height candidate: rejected; selected history unchanged");

    for height in 1..=4 {
        let selected = store.append(history.blocks()[height].clone())?;
        assert_eq!(selected.generation, height as u64);
        assert_eq!(selected.chain, fixture::main_expectation(height as u64));
        // Retain the just-confirmed checkpoint independently of the disk read.
        drop(store);
        store = AppendStoreV2::open_writer(&root, identity, selected)?;
        assert_eq!(store.chain()?, &fixture::main_chain(height as u64));
        println!(
            "Published and reopened height {} / generation {}",
            selected.chain.height, selected.generation
        );
    }

    assert_eq!(store.chain()?, &history);
    let selected = store.checkpoint()?;
    println!("Final tip: {}", selected.chain.tip_hash);
    println!("Final ledger root: {}", selected.chain.ledger_root);
    println!("Accounts, membership, expiry, approvals, withdrawals and budgets: reproduced");
    store = store.reconcile(selected)?;
    assert_eq!(store.chain()?, &history);
    drop(store);
    let inspection = AppendStoreV2::inspect(&root, identity, selected)?;
    assert_eq!(inspection.chain()?, &history);
    drop(inspection);
    println!("Append, checked reopening and clean-state reconciliation: VALID");
    println!(
        "Residual staging repair, branch replacement and live V2 networking remain later work."
    );
    Ok(())
}
