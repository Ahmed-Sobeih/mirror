//! Durable replacement of complete public synthetic histories in owned storage.
//! This example performs a real filesystem switch, then independently reopens it.

use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementStorageErrorV2, ReplacementStoreV2, StorageErrorV2,
};

#[path = "../tests/support_v2/mod.rs"]
mod fixture;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        // This is the freshly created synthetic example directory only.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!(
        "mirror-replacement-v2-demo-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
    ));
    fs::create_dir(&path)?;
    let scratch = Scratch(path);
    let root = scratch.0.join("selected-chain");

    // The fixture pins are frozen independently of the newly written store.
    // A real caller must retain its own complete branch/generation expectations.
    let identity = fixture::fixture_identity();
    let current_pin = ReplacementCheckpointV2 {
        generation: 0,
        chain: fixture::fork_expectation(),
    };
    let candidate_pin = ReplacementCheckpointV2 {
        generation: 1,
        chain: fixture::main_expectation(4),
    };
    let current = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let mut store = ReplacementStoreV2::initialize(&root, &current)?;
    assert_eq!(store.checkpoint()?, current_pin);

    println!("Mirror V2 — durable whole-branch replacement");
    println!("Public synthetic inputs; newly owned temporary directory");
    println!("Initialized fork: height 3 / local generation 0");
    assert!(matches!(
        ReplacementStoreV2::open_writer(&root, identity, current_pin),
        Err(ReplacementStorageErrorV2::Storage(StorageErrorV2::Busy))
    ));
    println!("Second writer: rejected while stable root ownership is held");

    // The equal-height main branch loses the tie, with no storage mutation.
    assert!(
        store
            .replace(identity, current_pin, &fixture::main_chain(3))
            .is_err()
    );
    assert_eq!(store.checkpoint()?, current_pin);
    assert!(!store.is_fenced());
    println!("Equal-height candidate: rejected; current branch retained");

    let selected = store.replace(identity, current_pin, &candidate)?;
    assert_eq!(selected, candidate_pin);
    assert_eq!(store.chain()?, &candidate);
    assert!(!root.join("generation-00000000000000000000").exists());
    assert!(!root.join("selection.pending").exists());
    assert_eq!(fs::read_dir(&root)?.count(), 2);
    println!("Published main: height 4 / local generation 1");
    println!("Candidate files synchronized; selector switched; old generation cleaned");

    drop(store);
    assert!(ReplacementStoreV2::open_writer(&root, identity, current_pin).is_err());
    let reopened = ReplacementStoreV2::open_writer(&root, identity, candidate_pin)?;
    assert_eq!(reopened.chain()?, &candidate);
    assert_eq!(
        reopened.chain()?.state_items_used(),
        candidate.state_items_used()
    );
    assert_eq!(reopened.chain()?.encoded_bytes(), candidate.encoded_bytes());
    println!("Fresh writer: complete selected MRB2 history reexecuted and matched");
    println!("Selected tip: {}", candidate_pin.chain.tip_hash);
    println!("Selected ledger root: {}", candidate_pin.chain.ledger_root);
    println!(
        "Active / pending / historical high-water: {} / {} / {}",
        reopened.chain()?.ledger().membership().active_count(),
        reopened.chain()?.ledger().membership().pending_count(),
        reopened.chain()?.ledger().membership().active_high_water(),
    );
    println!(
        "Encoded MRB2 bytes / cumulative state items: {} / {}",
        reopened.chain()?.encoded_bytes(),
        reopened.chain()?.state_items_used(),
    );
    let reconciled = reopened.reconcile(candidate_pin)?;
    assert_eq!(reconciled.chain()?, &candidate);
    drop(reconciled);
    let inspection = ReplacementStoreV2::inspect(&root, identity, candidate_pin)?;
    assert_eq!(inspection.chain()?, &candidate);
    drop(inspection);
    println!("Whole ledger, membership history and resource counters: reproduced");
    println!("Durable replacement and clean-state reconciliation: VALID");
    println!("Explicit recovery of interrupted staging/cleanup remains MB-03E2b");
    Ok(())
}
