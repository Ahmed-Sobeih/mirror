//! Explicit recovery of synthetic interrupted replacement layouts on disk.
//! This constructs authorized residue; it is not a physical power-loss test.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_chain::v2::ChainV2;
use mirror_storage::v2::{
    ReplacementCheckpointV2, ReplacementRecoveryActionV2 as Action,
    ReplacementRecoveryRequestV2 as Request, ReplacementStoreV2,
};

#[path = "../tests/support_v2/mod.rs"]
mod fixture;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        // Only this example's freshly created synthetic parent is removed.
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stage_partial_candidate(root: &Path, candidate: &ChainV2) -> Result<(), Box<dyn Error>> {
    let generation = root.join("generation-00000000000000000001");
    fs::create_dir(&generation)?;
    fs::create_dir(generation.join("blocks"))?;
    // Missing manifest and intermediate block entries are deliberately retained
    // as a synthetic example of unsynchronized directory-entry persistence.
    fs::write(generation.join("blocks/00000000000000000000.mrb2"), [])?;
    let bytes = candidate
        .block_at(4)
        .ok_or("fixture block four missing")?
        .encode(candidate.config())?;
    fs::write(
        generation.join("blocks/00000000000000000004.mrb2"),
        &bytes[..bytes.len() / 2],
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let parent = std::env::temp_dir().join(format!(
        "mirror-replacement-recovery-demo-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
    ));
    fs::create_dir(&parent)?;
    let scratch = Scratch(parent);
    let root = scratch.0.join("complete");
    let retained_root = scratch.0.join("retain-old");

    // Histories and endpoint pins originate from frozen independent fixtures,
    // never from the partial files being recovered.
    let identity = fixture::fixture_identity();
    let old_pin = ReplacementCheckpointV2 {
        generation: 0,
        chain: fixture::fork_expectation(),
    };
    let candidate_pin = ReplacementCheckpointV2 {
        generation: 1,
        chain: fixture::main_expectation(4),
    };
    let old = fixture::fork_chain();
    let candidate = fixture::main_chain(4);
    let request = Request::new(old_pin, candidate_pin, &old, &candidate)?;

    println!("Mirror V2 — explicit interrupted replacement recovery");
    println!("Public synthetic inputs; newly owned temporary directories");
    println!("Independent old/new full histories: fork height 3 / main height 4");

    drop(ReplacementStoreV2::initialize(&root, &old)?);
    stage_partial_candidate(&root, &candidate)?;
    assert!(ReplacementStoreV2::open_writer(&root, identity, old_pin).is_err());
    println!("Ordinary opening rejects the simulated partial candidate unchanged");

    let recovered =
        ReplacementStoreV2::recover_interrupted(&root, identity, &request, Action::Complete)?;
    assert_eq!(recovered.checkpoint()?, candidate_pin);
    assert_eq!(recovered.chain()?, &candidate);
    assert!(!root.join("generation-00000000000000000000").exists());
    assert!(!root.join("selection.pending").exists());
    assert_eq!(fs::read_dir(&root)?.count(), 2);
    println!("Complete: canonical candidate files finished, validated and selected");
    println!("Exact old generation retired after candidate publication");
    drop(recovered);

    let reopened = ReplacementStoreV2::open_writer(&root, identity, candidate_pin)?;
    assert_eq!(reopened.chain()?, &candidate);
    assert_eq!(reopened.chain()?.encoded_bytes(), candidate.encoded_bytes());
    assert_eq!(
        reopened.chain()?.state_items_used(),
        candidate.state_items_used()
    );
    println!("Fresh writer: complete ledger and personhood history reproduced");
    // Consuming recovery keeps this same writer's root ownership continuously.
    let repeated = reopened.recover(&request, Action::Complete)?;
    assert_eq!(repeated.chain()?, &candidate);
    drop(repeated);
    assert!(
        ReplacementStoreV2::recover_interrupted(&root, identity, &request, Action::RetainOld,)
            .is_err()
    );
    println!("Repeated completion succeeds; rollback of selected candidate rejects");

    drop(ReplacementStoreV2::initialize(&retained_root, &old)?);
    stage_partial_candidate(&retained_root, &candidate)?;
    let retained = ReplacementStoreV2::recover_interrupted(
        &retained_root,
        identity,
        &request,
        Action::RetainOld,
    )?;
    assert_eq!(retained.checkpoint()?, old_pin);
    assert_eq!(retained.chain()?, &old);
    assert!(
        !retained_root
            .join("generation-00000000000000000001")
            .exists()
    );
    assert_eq!(fs::read_dir(&retained_root)?.count(), 2);
    println!("RetainOld: authorized partial candidate removed; old branch unchanged");
    println!("Selected candidate tip: {}", candidate_pin.chain.tip_hash);
    println!(
        "Selected candidate ledger root: {}",
        candidate_pin.chain.ledger_root
    );
    println!(
        "Candidate active / pending / historical high-water: {} / {} / {}",
        candidate.ledger().membership().active_count(),
        candidate.ledger().membership().pending_count(),
        candidate.ledger().membership().active_high_water(),
    );
    println!("Explicit completion, retention and clean reopening: VALID");
    println!("Missing/corrupt selectors and unknown files still require rejection");
    Ok(())
}
