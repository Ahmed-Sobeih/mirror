//! Public-API checks of independently pinned, operator-owned restart authority.
//! Every altered filesystem object belongs to a newly created synthetic fixture.
#![cfg(target_os = "linux")]

mod support_control_v2;

use std::{
    fs::{self, File},
    os::unix::fs::{MetadataExt, symlink},
};

use mirror_chain::v2::{ChainV2, ForkChoiceV2, ManifestIdentityV2};
use mirror_crypto::Hash256;
use mirror_node::v2::records::{ControlRecordV2, IntentRecordV2, MAX_INTENT_BYTES_V2};
use mirror_node::v2::{NodeControlV2, NodeStatusV2};
use mirror_storage::v2::{ReplacementRecoveryActionV2, ReplacementStoreV2};

use support_control_v2::{
    Scratch, assert_complete_state, checkpoint, control_root,
    history::{fork_chain, independent_identity, main_chain},
    preferred_fork, received, snapshot,
};

fn storage(scratch: &Scratch, chain: &ChainV2) {
    drop(ReplacementStoreV2::initialize(scratch.root(), chain).unwrap());
}

fn initialized(scratch: &Scratch, chain: &ChainV2) -> NodeControlV2 {
    storage(scratch, chain);
    NodeControlV2::initialize(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
        checkpoint(chain, 0),
    )
    .unwrap()
}

fn reopen(scratch: &Scratch) -> NodeControlV2 {
    NodeControlV2::open_writer(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
    )
    .unwrap()
}

fn status(scratch: &Scratch) -> NodeStatusV2 {
    NodeControlV2::status(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
    )
    .unwrap()
}

fn intent(scratch: &Scratch, old: &ChainV2, candidate: &ChainV2) -> IntentRecordV2 {
    let record = IntentRecordV2::new(
        independent_identity(),
        checkpoint(old, 0),
        old.clone(),
        candidate.clone(),
    )
    .unwrap();
    fs::write(
        control_root(scratch).join("intent.v2"),
        record.encode().unwrap(),
    )
    .unwrap();
    record
}

fn recover(
    scratch: &Scratch,
    action: ReplacementRecoveryActionV2,
) -> Result<NodeControlV2, mirror_node::v2::NodeControlErrorV2> {
    NodeControlV2::recover(
        control_root(scratch),
        scratch.root(),
        independent_identity(),
        action,
    )
}

fn assert_readers_reject_without_changes(scratch: &Scratch) {
    let before = snapshot(&scratch.0);
    assert!(
        NodeControlV2::status(
            control_root(scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert!(
        NodeControlV2::open_writer(
            control_root(scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert!(recover(scratch, ReplacementRecoveryActionV2::Complete).is_err());
    assert!(recover(scratch, ReplacementRecoveryActionV2::RetainOld).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn missing_status_does_not_create_control_and_open_never_initializes_it() {
    let scratch = Scratch::new("node-missing");
    storage(&scratch, &main_chain(1));
    let before = snapshot(&scratch.0);
    assert_eq!(status(&scratch), NodeStatusV2::Missing);
    assert!(!control_root(&scratch).exists());
    assert!(
        NodeControlV2::open_writer(
            control_root(&scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn explicit_control_initialization_attaches_existing_store_and_clean_restart_replays_it() {
    let scratch = Scratch::new("node-clean");
    let chain = main_chain(3);
    let owner = initialized(&scratch, &chain);
    assert_eq!(owner.checkpoint(), checkpoint(&chain, 0));
    assert_complete_state(owner.chain(), &chain);
    drop(owner);
    let before = snapshot(&scratch.0);
    assert_eq!(
        status(&scratch),
        NodeStatusV2::Clean {
            checkpoint: checkpoint(&chain, 0)
        }
    );
    assert_eq!(snapshot(&scratch.0), before);
    let reopened = reopen(&scratch);
    assert_complete_state(reopened.chain(), &chain);
    assert_eq!(reopened.checkpoint(), checkpoint(&chain, 0));
}

#[test]
fn initialization_never_creates_a_missing_store() {
    let scratch = Scratch::new("node-no-store");
    assert!(
        NodeControlV2::initialize(
            control_root(&scratch),
            scratch.root(),
            independent_identity(),
            checkpoint(&main_chain(1), 0)
        )
        .is_err()
    );
    assert!(!scratch.root().exists());
}

#[test]
fn repeated_initialization_preserves_existing_control_and_selected_history() {
    let scratch = Scratch::new("node-repeat-init");
    let chain = main_chain(1);
    drop(initialized(&scratch, &chain));
    let before = snapshot(&scratch.0);
    assert!(
        NodeControlV2::initialize(
            control_root(&scratch),
            scratch.root(),
            independent_identity(),
            checkpoint(&chain, 0)
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn independent_identity_and_wrong_full_initial_pins_reject_before_control_publication() {
    let chain = main_chain(3);
    for case in 0..5 {
        let scratch = Scratch::new("node-wrong-init-pins");
        storage(&scratch, &chain);
        fs::create_dir(control_root(&scratch)).unwrap();
        let mut identity = independent_identity();
        let mut expected = checkpoint(&chain, 0);
        match case {
            0 => identity.config_hash = Hash256::from_bytes([9; 32]),
            1 => identity.genesis_hash = Hash256::from_bytes([8; 32]),
            2 => expected.generation = 1,
            3 => expected.chain.tip_hash = Hash256::from_bytes([7; 32]),
            _ => expected.chain.ledger_root = Hash256::from_bytes([6; 32]),
        }
        let before = snapshot(&scratch.0);
        assert!(
            NodeControlV2::initialize(control_root(&scratch), scratch.root(), identity, expected)
                .is_err()
        );
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn independently_wrong_network_rejects_clean_status_open_and_recovery() {
    let scratch = Scratch::new("node-wrong-network");
    drop(initialized(&scratch, &main_chain(1)));
    let before = snapshot(&scratch.0);
    for identity in [
        ManifestIdentityV2 {
            config_hash: Hash256::from_bytes([1; 32]),
            ..independent_identity()
        },
        ManifestIdentityV2 {
            genesis_hash: Hash256::from_bytes([2; 32]),
            ..independent_identity()
        },
    ] {
        assert!(NodeControlV2::status(control_root(&scratch), scratch.root(), identity).is_err());
        assert!(
            NodeControlV2::open_writer(control_root(&scratch), scratch.root(), identity).is_err()
        );
        assert!(
            NodeControlV2::recover(
                control_root(&scratch),
                scratch.root(),
                identity,
                ReplacementRecoveryActionV2::Complete
            )
            .is_err()
        );
    }
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn controller_owns_both_roots_and_excludes_competing_controller_and_direct_store_writer() {
    let scratch = Scratch::new("node-ownership");
    let chain = main_chain(1);
    let owner = initialized(&scratch, &chain);
    let before = snapshot(&scratch.0);
    assert!(
        NodeControlV2::open_writer(
            control_root(&scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert!(
        NodeControlV2::status(
            control_root(&scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert!(
        ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&chain, 0)
        )
        .is_err()
    );
    assert!(
        ReplacementStoreV2::inspect(
            scratch.root(),
            independent_identity(),
            checkpoint(&chain, 0)
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
    drop(owner);
    drop(reopen(&scratch));
}

#[test]
fn tied_shorter_and_identical_candidates_leave_all_control_and_store_bytes_unchanged() {
    let old = main_chain(3);
    for candidate in [old.clone(), main_chain(1), fork_chain()] {
        let scratch = Scratch::new("node-keep");
        let owner = initialized(&scratch, &old);
        let before = snapshot(&scratch.0);
        let outcome = owner.adopt(received(&candidate)).unwrap();
        assert_eq!(outcome.report().choice, ForkChoiceV2::KeepCurrent);
        assert_eq!(outcome.report().selected, checkpoint(&old, 0));
        let owner = outcome.into_control();
        assert_complete_state(owner.chain(), &old);
        assert_eq!(snapshot(&scratch.0), before);
        assert!(!control_root(&scratch).join("intent.v2").exists());
    }
}

#[test]
fn preferred_membership_fork_publishes_complete_state_and_retires_durable_intent() {
    let scratch = Scratch::new("node-membership-fork");
    let old = main_chain(3);
    let candidate = preferred_fork();
    assert_eq!(
        old.ledger().account_bytes().unwrap(),
        candidate.ledger().account_bytes().unwrap()
    );
    assert_ne!(
        old.ledger().membership().encode().unwrap(),
        candidate.ledger().membership().encode().unwrap()
    );
    let owner = initialized(&scratch, &old);
    let outcome = owner.adopt(received(&candidate)).unwrap();
    assert_eq!(outcome.report().choice, ForkChoiceV2::PreferCandidate);
    assert_eq!(outcome.report().selected, checkpoint(&candidate, 1));
    let owner = outcome.into_control();
    assert_complete_state(owner.chain(), &candidate);
    drop(owner);
    assert_eq!(fs::read_dir(control_root(&scratch)).unwrap().count(), 1);
    assert!(control_root(&scratch).join("confirmed.v2").is_file());
    assert_eq!(
        status(&scratch),
        NodeStatusV2::Clean {
            checkpoint: checkpoint(&candidate, 1)
        }
    );
    assert_complete_state(reopen(&scratch).chain(), &candidate);
}

#[test]
fn valid_intent_status_is_read_only_and_ordinary_open_requires_explicit_recovery() {
    let scratch = Scratch::new("node-intent-status");
    let old = main_chain(3);
    let candidate = preferred_fork();
    drop(initialized(&scratch, &old));
    let record = intent(&scratch, &old, &candidate);
    let before = snapshot(&scratch.0);
    assert_eq!(
        status(&scratch),
        NodeStatusV2::RecoveryRequired {
            confirmed: record.old(),
            old: record.old(),
            candidate: record.candidate()
        }
    );
    assert!(
        NodeControlV2::open_writer(
            control_root(&scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn explicit_complete_from_durable_intent_replays_and_publishes_both_membership_histories() {
    let scratch = Scratch::new("node-complete");
    let old = main_chain(3);
    let candidate = preferred_fork();
    drop(initialized(&scratch, &old));
    intent(&scratch, &old, &candidate);
    let owner = recover(&scratch, ReplacementRecoveryActionV2::Complete).unwrap();
    assert_complete_state(owner.chain(), &candidate);
    assert_eq!(owner.checkpoint(), checkpoint(&candidate, 1));
    drop(owner);
    assert_eq!(
        status(&scratch),
        NodeStatusV2::Clean {
            checkpoint: checkpoint(&candidate, 1)
        }
    );
    assert_eq!(fs::read_dir(control_root(&scratch)).unwrap().count(), 1);
}

#[test]
fn explicit_retain_old_retires_intent_without_replacing_selected_history() {
    let scratch = Scratch::new("node-retain");
    let old = main_chain(3);
    let candidate = preferred_fork();
    drop(initialized(&scratch, &old));
    intent(&scratch, &old, &candidate);
    let before = snapshot(&scratch.root());
    let owner = recover(&scratch, ReplacementRecoveryActionV2::RetainOld).unwrap();
    assert_complete_state(owner.chain(), &old);
    assert_eq!(owner.checkpoint(), checkpoint(&old, 0));
    assert_eq!(snapshot(&scratch.root()), before);
    assert!(!control_root(&scratch).join("intent.v2").exists());
}

#[test]
fn already_selected_candidate_cannot_be_retained_as_old_even_with_old_confirmation() {
    let scratch = Scratch::new("node-no-rollback");
    let old = main_chain(3);
    let candidate = preferred_fork();
    drop(initialized(&scratch, &old));
    intent(&scratch, &old, &candidate);
    let mut store = ReplacementStoreV2::open_writer(
        scratch.root(),
        independent_identity(),
        checkpoint(&old, 0),
    )
    .unwrap();
    store
        .replace(independent_identity(), checkpoint(&old, 0), &candidate)
        .unwrap();
    drop(store);
    let before = snapshot(&scratch.0);
    assert!(recover(&scratch, ReplacementRecoveryActionV2::RetainOld).is_err());
    assert_eq!(snapshot(&scratch.0), before);
    let owner = recover(&scratch, ReplacementRecoveryActionV2::Complete).unwrap();
    assert_complete_state(owner.chain(), &candidate);
    assert_eq!(owner.checkpoint(), checkpoint(&candidate, 1));
}

#[test]
fn confirmed_candidate_with_old_selected_store_is_inconsistent_and_preserved() {
    let scratch = Scratch::new("node-confirmed-new-old-store");
    let old = main_chain(3);
    let candidate = preferred_fork();
    drop(initialized(&scratch, &old));
    let record = intent(&scratch, &old, &candidate);
    fs::write(
        control_root(&scratch).join("confirmed.v2"),
        record.candidate_record().encode(),
    )
    .unwrap();
    let before = snapshot(&scratch.0);
    assert!(recover(&scratch, ReplacementRecoveryActionV2::Complete).is_err());
    assert!(recover(&scratch, ReplacementRecoveryActionV2::RetainOld).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn confirmation_staging_with_old_selected_store_cannot_authorize_a_new_publication() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for len in [0, 7, 463, 464] {
        let scratch = Scratch::new("node-pending-confirm-old-store");
        drop(initialized(&scratch, &old));
        let record = intent(&scratch, &old, &candidate);
        fs::write(
            control_root(&scratch).join("confirmed.pending"),
            &record.candidate_record().encode()[..len],
        )
        .unwrap();
        let before = snapshot(&scratch.0);
        assert!(recover(&scratch, ReplacementRecoveryActionV2::Complete).is_err());
        assert!(recover(&scratch, ReplacementRecoveryActionV2::RetainOld).is_err());
        assert_eq!(snapshot(&scratch.0), before);
    }
}

#[test]
fn mismatched_confirmed_endpoint_or_canonical_metadata_rejects_intent() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for replacement in [
        ControlRecordV2::from_chain(independent_identity(), &main_chain(2), 0).unwrap(),
        ControlRecordV2::from_chain(independent_identity(), &old, 1).unwrap(),
    ] {
        let scratch = Scratch::new("node-conflicting-confirmation");
        drop(initialized(&scratch, &old));
        intent(&scratch, &old, &candidate);
        fs::write(
            control_root(&scratch).join("confirmed.v2"),
            replacement.encode(),
        )
        .unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn corrupt_truncated_overlong_or_unsupported_confirmed_records_are_preserved() {
    let original = ControlRecordV2::from_chain(independent_identity(), &main_chain(1), 0)
        .unwrap()
        .encode();
    for case in 0..5 {
        let scratch = Scratch::new("node-invalid-confirmed");
        drop(initialized(&scratch, &main_chain(1)));
        let mut bytes = original.to_vec();
        match case {
            0 => bytes.clear(),
            1 => {
                bytes.pop();
            }
            2 => bytes.push(0),
            3 => bytes[463] ^= 1,
            _ => bytes[4] = 3,
        }
        fs::write(control_root(&scratch).join("confirmed.v2"), bytes).unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn corrupt_truncated_and_overlong_intent_never_authorize_store_changes() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for case in 0..4 {
        let scratch = Scratch::new("node-invalid-intent");
        drop(initialized(&scratch, &old));
        let mut bytes = intent(&scratch, &old, &candidate).encode().unwrap();
        match case {
            0 => bytes.clear(),
            1 => {
                bytes.pop();
            }
            2 => bytes.push(0),
            _ => {
                let last = bytes.len() - 1;
                bytes[last] ^= 1;
            }
        }
        fs::write(control_root(&scratch).join("intent.v2"), bytes).unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn oversized_sparse_intent_is_rejected_before_store_mutation() {
    let scratch = Scratch::new("node-oversized-intent");
    drop(initialized(&scratch, &main_chain(3)));
    let path = control_root(&scratch).join("intent.v2");
    File::create(&path)
        .unwrap()
        .set_len(MAX_INTENT_BYTES_V2 as u64 + 1)
        .unwrap();
    let before_store = snapshot(&scratch.root());
    let before = fs::metadata(&path).unwrap();
    assert!(
        NodeControlV2::status(
            control_root(&scratch),
            scratch.root(),
            independent_identity()
        )
        .is_err()
    );
    assert!(recover(&scratch, ReplacementRecoveryActionV2::Complete).is_err());
    let after = fs::metadata(&path).unwrap();
    assert_eq!(
        (after.len(), after.ino(), after.mtime(), after.mtime_nsec()),
        (
            before.len(),
            before.ino(),
            before.mtime(),
            before.mtime_nsec()
        )
    );
    assert_eq!(snapshot(&scratch.root()), before_store);
}

#[test]
fn mismatched_or_overlong_confirmation_staging_is_rejected_before_recovery() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for overlong in [false, true] {
        let scratch = Scratch::new("node-invalid-confirmation-staging");
        drop(initialized(&scratch, &old));
        let record = intent(&scratch, &old, &candidate);
        let mut store = ReplacementStoreV2::open_writer(
            scratch.root(),
            independent_identity(),
            checkpoint(&old, 0),
        )
        .unwrap();
        store
            .replace(independent_identity(), checkpoint(&old, 0), &candidate)
            .unwrap();
        drop(store);
        let mut bytes = record.candidate_record().encode().to_vec();
        if overlong {
            bytes.push(0);
        } else {
            bytes[20] ^= 1;
        }
        fs::write(control_root(&scratch).join("confirmed.pending"), bytes).unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn intent_staging_is_preserved_even_when_it_contains_a_complete_valid_record() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for complete in [false, true] {
        let scratch = Scratch::new("node-intent-staging");
        drop(initialized(&scratch, &old));
        let record = IntentRecordV2::new(
            independent_identity(),
            checkpoint(&old, 0),
            old.clone(),
            candidate.clone(),
        )
        .unwrap();
        let bytes = if complete {
            record.encode().unwrap()
        } else {
            b"MNI2".to_vec()
        };
        fs::write(control_root(&scratch).join("intent.pending"), bytes).unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn orphan_confirmation_staging_and_partial_control_initialization_are_not_repaired() {
    for initialized_control in [false, true] {
        let scratch = Scratch::new("node-orphan-confirmation");
        if initialized_control {
            drop(initialized(&scratch, &main_chain(1)));
        } else {
            storage(&scratch, &main_chain(1));
            fs::create_dir(control_root(&scratch)).unwrap();
        }
        fs::write(control_root(&scratch).join("confirmed.pending"), b"MNC2").unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn unknown_control_names_and_subdirectories_are_not_traversed_or_removed() {
    for directory in [false, true] {
        let scratch = Scratch::new("node-unknown-control");
        drop(initialized(&scratch, &main_chain(1)));
        let unknown = control_root(&scratch).join("foreign");
        if directory {
            fs::create_dir(&unknown).unwrap();
            fs::write(unknown.join("keep"), b"synthetic unknown data").unwrap();
        } else {
            fs::write(&unknown, b"synthetic unknown data").unwrap();
        }
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn symbolic_and_hard_linked_control_files_are_rejected_without_touching_targets() {
    for hardlink in [false, true] {
        let scratch = Scratch::new("node-unsafe-file");
        drop(initialized(&scratch, &main_chain(1)));
        let confirmed = control_root(&scratch).join("confirmed.v2");
        let target = scratch.0.join("external-confirmed");
        fs::rename(&confirmed, &target).unwrap();
        if hardlink {
            fs::hard_link(&target, &confirmed).unwrap();
        } else {
            symlink(&target, &confirmed).unwrap();
        }
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn unsafe_intent_and_confirmation_staging_files_are_preserved() {
    let old = main_chain(3);
    let candidate = preferred_fork();
    for name in ["intent.v2", "confirmed.pending"] {
        for hardlink in [false, true] {
            let scratch = Scratch::new("node-unsafe-intent");
            drop(initialized(&scratch, &old));
            let record = intent(&scratch, &old, &candidate);
            let path = control_root(&scratch).join(name);
            let target = scratch.0.join("external-file");
            if name == "intent.v2" {
                fs::rename(&path, &target).unwrap();
            } else {
                fs::write(&target, record.candidate_record().encode()).unwrap();
            }
            if hardlink {
                fs::hard_link(&target, &path).unwrap();
            } else {
                symlink(&target, &path).unwrap();
            }
            assert_readers_reject_without_changes(&scratch);
        }
    }
}

#[test]
fn symbolic_control_or_store_root_is_rejected_instead_of_followed() {
    for replace_control in [false, true] {
        let scratch = Scratch::new("node-symlink-root");
        drop(initialized(&scratch, &main_chain(1)));
        let path = if replace_control {
            control_root(&scratch)
        } else {
            scratch.root()
        };
        let target = scratch.0.join("real-root");
        fs::rename(&path, &target).unwrap();
        symlink(&target, &path).unwrap();
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn identical_or_nested_root_pairs_are_rejected_without_mutation() {
    let scratch = Scratch::new("node-overlapping-roots");
    let chain = main_chain(1);
    drop(initialized(&scratch, &chain));
    let before = snapshot(&scratch.0);
    for control in [
        scratch.root(),
        scratch.root().join("control"),
        scratch.0.clone(),
    ] {
        assert!(
            NodeControlV2::initialize(
                &control,
                scratch.root(),
                independent_identity(),
                checkpoint(&chain, 0)
            )
            .is_err()
        );
        assert!(
            NodeControlV2::open_writer(&control, scratch.root(), independent_identity()).is_err()
        );
    }
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn clean_control_cannot_hide_corrupt_or_unexpected_selected_store_contents() {
    for case in 0..3 {
        let scratch = Scratch::new("node-invalid-store");
        drop(initialized(&scratch, &main_chain(1)));
        match case {
            0 => fs::write(scratch.root().join("unknown"), b"preserve").unwrap(),
            1 => {
                let path = scratch.root().join("selection.v2");
                let mut bytes = fs::read(&path).unwrap();
                bytes[279] ^= 1;
                fs::write(path, bytes).unwrap();
            }
            _ => fs::write(scratch.root().join("selection.pending"), b"").unwrap(),
        }
        assert_readers_reject_without_changes(&scratch);
    }
}

#[test]
fn stale_control_checkpoint_does_not_choose_the_higher_store_generation() {
    let scratch = Scratch::new("node-stale-confirmed");
    let old = main_chain(1);
    let candidate = main_chain(4);
    drop(initialized(&scratch, &old));
    let mut store = ReplacementStoreV2::open_writer(
        scratch.root(),
        independent_identity(),
        checkpoint(&old, 0),
    )
    .unwrap();
    store
        .replace(independent_identity(), checkpoint(&old, 0), &candidate)
        .unwrap();
    drop(store);
    assert_readers_reject_without_changes(&scratch);
}

#[test]
fn changed_control_bytes_under_owned_controller_reject_before_new_store_mutation() {
    let scratch = Scratch::new("node-control-changed");
    let old = main_chain(1);
    let owner = initialized(&scratch, &old);
    fs::write(control_root(&scratch).join("unexpected"), b"preserve").unwrap();
    let before = snapshot(&scratch.0);
    assert!(owner.adopt(received(&main_chain(4))).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}

#[test]
fn replacing_the_owned_control_root_inode_is_detected_before_adoption() {
    let scratch = Scratch::new("node-control-root-replaced");
    let old = main_chain(1);
    let owner = initialized(&scratch, &old);
    let original = scratch.0.join("retained-control");
    fs::rename(control_root(&scratch), &original).unwrap();
    fs::create_dir(control_root(&scratch)).unwrap();
    fs::copy(
        original.join("confirmed.v2"),
        control_root(&scratch).join("confirmed.v2"),
    )
    .unwrap();
    let before = snapshot(&scratch.0);
    assert!(owner.adopt(received(&main_chain(4))).is_err());
    assert_eq!(snapshot(&scratch.0), before);
}
