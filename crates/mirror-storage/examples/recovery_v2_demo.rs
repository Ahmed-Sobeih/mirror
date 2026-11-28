//! Explicit recovery using public synthetic fixtures and new owned paths only.
//!
//! This creates known partial-write shapes deliberately. It does not simulate
//! physical power loss or recover the user's existing chain directories.
use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_storage::v2::{
    AppendCheckpointV2, AppendRecoveryActionV2, AppendRecoveryRequestV2, AppendSelectionV2,
    AppendStoreV2,
};

#[path = "../tests/support_v2/mod.rs"]
mod fixture;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        // This parent was exclusively created by this invocation. Production
        // recovery never recursively removes a directory or unexplained data.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn checkpoint(height: u64, generation: u64) -> AppendCheckpointV2 {
    AppendCheckpointV2 {
        generation,
        chain: fixture::main_expectation(height),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::temp_dir().join(format!(
        "mirror-recovery-v2-demo-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
    ));
    fs::create_dir(&path)?;
    let scratch = Scratch(path);
    let root = scratch.0.join("selected-chain");
    let identity = fixture::fixture_identity();
    let history = fixture::main_chain(2);
    let first_request = AppendRecoveryRequestV2::new(
        checkpoint(0, 0),
        checkpoint(1, 1),
        history.blocks()[1].clone(),
    )?;
    let second_request = AppendRecoveryRequestV2::new(
        checkpoint(1, 1),
        checkpoint(2, 2),
        history.blocks()[2].clone(),
    )?;
    drop(AppendStoreV2::initialize(&root, &fixture::main_chain(0))?);

    println!("Mirror V2 — explicit interrupted-append recovery");
    println!("Recovery authority: independently retained parent, candidate and exact child");

    // A synthetic interruption while writing the next block: no pending marker
    // could have been prepared before this block became complete.
    let first_path = root.join("blocks/00000000000000000001.mrb2");
    let first_bytes = fixture::fixture_bytes("block1_block");
    let partial = &first_bytes[..first_bytes.len() / 2];
    fs::write(&first_path, partial)?;
    assert!(AppendStoreV2::open_writer(&root, identity, checkpoint(0, 0)).is_err());
    assert_eq!(fs::read(&first_path)?, partial);
    println!("Normal opening: rejected the partial tail and preserved its bytes");

    let mut store = AppendStoreV2::recover_interrupted(
        &root,
        identity,
        &first_request,
        AppendRecoveryActionV2::Complete,
    )?;
    assert_eq!(store.chain()?, &fixture::main_chain(1));
    assert_eq!(store.checkpoint()?, checkpoint(1, 1));
    assert_eq!(fs::read(&first_path)?, first_bytes);
    assert!(!root.join("selection.pending").exists());
    println!("Complete: extended the authorized prefix and selected height 1 / generation 1");

    // Retrying the same operation cannot add a second block or generation.
    store = store.recover(&first_request, AppendRecoveryActionV2::Complete)?;
    assert_eq!(store.checkpoint()?, checkpoint(1, 1));
    println!("Repeated completion: validated the same selection without a second advance");

    // Construct another known interruption shape inside this synthetic owned
    // directory: full next block, followed by only a prefix of its pending marker.
    // The live append detects the inconsistent disk layout and fences itself.
    let second_path = root.join("blocks/00000000000000000002.mrb2");
    fs::write(&second_path, fixture::fixture_bytes("block2_block"))?;
    let next_marker = AppendSelectionV2::from_chain(&history, 2)?.encode();
    fs::write(root.join("selection.pending"), &next_marker[..120])?;
    assert!(store.append(history.blocks()[2].clone()).is_err());
    assert!(store.is_fenced());
    assert!(store.chain().is_err());
    println!("Incomplete second preparation: existing writer fenced further updates");

    store = store.recover(&second_request, AppendRecoveryActionV2::RetainParent)?;
    assert_eq!(store.chain()?, &fixture::main_chain(1));
    assert_eq!(store.checkpoint()?, checkpoint(1, 1));
    assert!(!second_path.exists());
    assert!(!root.join("selection.pending").exists());
    println!("RetainParent: removed only authorized staging and kept height 1 / generation 1");

    let selected = store.append(history.blocks()[2].clone())?;
    assert_eq!(selected, checkpoint(2, 2));
    assert_eq!(store.chain()?, &history);
    drop(store);
    let reopened = AppendStoreV2::open_writer(&root, identity, selected)?;
    assert_eq!(reopened.chain()?, &history);
    println!("Next ordinary append and checked reopening: VALID");
    println!("Final tip: {}", selected.chain.tip_hash);
    println!("Final ledger root: {}", selected.chain.ledger_root);
    println!("Complete accounts, membership and resource counters: reproduced");
    println!("Unknown/corrupt files and unsupported layouts remain explicit rejection cases.");
    Ok(())
}
