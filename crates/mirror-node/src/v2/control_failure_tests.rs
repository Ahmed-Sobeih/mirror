//! Deterministic returned-error and changed-artifact boundaries for node control.
//! All callbacks are private test hooks; no production caller can enable them.
#![cfg(target_os = "linux")]

#[path = "../../tests/support_control_v2/mod.rs"]
mod support;

use std::{fs, io};

use super::control_process_tests::PROCESSES;
use super::*;
use support::{
    Scratch, assert_complete_state, checkpoint, control_root,
    history::{independent_identity, main_chain},
    preferred_fork, received, snapshot,
};

fn prepared(label: &str) -> (Scratch, NodeControlV2) {
    let scratch = Scratch::new(label);
    let old = main_chain(3);
    drop(ReplacementStoreV2::initialize(scratch.root(), &old).unwrap());
    let node = NodeControlV2::initialize(
        control_root(&scratch),
        scratch.root(),
        independent_identity(),
        checkpoint(&old, 0),
    )
    .unwrap();
    (scratch, node)
}

fn injected() -> NodeControlErrorV2 {
    NodeControlErrorV2::Io(io::Error::other("injected completed-boundary failure"))
}

fn pending_intent(label: &str) -> Scratch {
    let (scratch, node) = prepared(label);
    let result = node.adopt_inner(received(&preferred_fork()), |boundary| {
        if boundary == ControlBoundaryV2::IntentDirectorySynced {
            Err(injected())
        } else {
            Ok(())
        }
    });
    assert!(matches!(result, Err(NodeControlErrorV2::Io(_))));
    assert!(control_root(&scratch).join(INTENT_NAME).is_file());
    scratch
}

fn assert_released(scratch: &Scratch, candidate: bool) {
    let lock = ControlLock::acquire(&control_root(scratch), true).unwrap();
    let chain = if candidate {
        preferred_fork()
    } else {
        main_chain(3)
    };
    let store = ReplacementStoreV2::open_writer(
        scratch.root(),
        independent_identity(),
        checkpoint(&chain, u64::from(candidate)),
    )
    .unwrap();
    assert_complete_state(store.chain().unwrap(), &chain);
    drop(store);
    drop(lock);
}

fn explicit_retry_or_clean_open(
    scratch: &Scratch,
    action: ReplacementRecoveryActionV2,
    candidate: bool,
) {
    let control = control_root(scratch);
    let node = if control.join(INTENT_NAME).exists() {
        assert!(matches!(
            NodeControlV2::status(&control, scratch.root(), independent_identity()).unwrap(),
            NodeStatusV2::RecoveryRequired { .. }
        ));
        assert!(
            NodeControlV2::open_writer(&control, scratch.root(), independent_identity()).is_err()
        );
        NodeControlV2::recover(&control, scratch.root(), independent_identity(), action).unwrap()
    } else {
        NodeControlV2::open_writer(&control, scratch.root(), independent_identity()).unwrap()
    };
    let chain = if candidate {
        preferred_fork()
    } else {
        main_chain(3)
    };
    assert_eq!(node.checkpoint(), checkpoint(&chain, u64::from(candidate)));
    assert_complete_state(node.chain(), &chain);
    assert_eq!(fs::read_dir(&control).unwrap().count(), 1);
    assert!(control.join(CONFIRMED_NAME).is_file());
}

#[test]
fn adoption_returned_errors_release_ownership_and_preserve_phase_appropriate_authority() {
    let _serial = PROCESSES.lock().unwrap();
    let phases = [
        ControlBoundaryV2::IntentCreated,
        ControlBoundaryV2::IntentPrefixWritten,
        ControlBoundaryV2::IntentWritten,
        ControlBoundaryV2::IntentFileSynced,
        ControlBoundaryV2::IntentValidated,
        ControlBoundaryV2::IntentPublished,
        ControlBoundaryV2::IntentDirectorySynced,
        ControlBoundaryV2::StorePublicationCompleted,
        ControlBoundaryV2::ConfirmedCreated,
        ControlBoundaryV2::ConfirmedPrefixWritten,
        ControlBoundaryV2::ConfirmedWritten,
        ControlBoundaryV2::ConfirmedFileSynced,
        ControlBoundaryV2::ConfirmedPublished,
        ControlBoundaryV2::ConfirmedDirectorySynced,
        ControlBoundaryV2::IntentRemoved,
        ControlBoundaryV2::IntentRetirementSynced,
    ];
    for (index, target) in phases.into_iter().enumerate() {
        let (scratch, node) = prepared("node-returned-adoption-error");
        let before_store = snapshot(&scratch.root());
        let mut reached = false;
        let result = node.adopt_inner(received(&preferred_fork()), |boundary| {
            if boundary == target {
                reached = true;
                Err(injected())
            } else {
                Ok(())
            }
        });
        assert!(reached, "missing hook {target:?}");
        assert!(
            matches!(result, Err(NodeControlErrorV2::Io(_))),
            "wrong result at {target:?}"
        );
        let candidate = index >= 7;
        if !candidate {
            assert_eq!(
                snapshot(&scratch.root()),
                before_store,
                "store mutated at {target:?}"
            );
        }
        assert_released(&scratch, candidate);
        let control = control_root(&scratch);
        if index < 5 {
            assert!(control.join(INTENT_PENDING_NAME).is_file());
            assert!(!control.join(INTENT_NAME).exists());
            let before = snapshot(&scratch.0);
            assert!(
                NodeControlV2::status(&control, scratch.root(), independent_identity()).is_err()
            );
            for action in [
                ReplacementRecoveryActionV2::Complete,
                ReplacementRecoveryActionV2::RetainOld,
            ] {
                assert!(
                    NodeControlV2::recover(
                        &control,
                        scratch.root(),
                        independent_identity(),
                        action
                    )
                    .is_err()
                );
            }
            assert_eq!(snapshot(&scratch.0), before);
        } else {
            explicit_retry_or_clean_open(&scratch, ReplacementRecoveryActionV2::Complete, true);
        }
    }
}

#[test]
fn recovery_returned_errors_allow_only_explicit_phase_consistent_retry() {
    let _serial = PROCESSES.lock().unwrap();
    let complete = [
        ControlBoundaryV2::IntentResynchronized,
        ControlBoundaryV2::StorePublicationCompleted,
        ControlBoundaryV2::ConfirmedCreated,
        ControlBoundaryV2::ConfirmedPrefixWritten,
        ControlBoundaryV2::ConfirmedWritten,
        ControlBoundaryV2::ConfirmedFileSynced,
        ControlBoundaryV2::ConfirmedPublished,
        ControlBoundaryV2::ConfirmedDirectorySynced,
        ControlBoundaryV2::IntentRemoved,
        ControlBoundaryV2::IntentRetirementSynced,
    ];
    let retain = [
        ControlBoundaryV2::IntentResynchronized,
        ControlBoundaryV2::StorePublicationCompleted,
        ControlBoundaryV2::ConfirmedDirectorySynced,
        ControlBoundaryV2::IntentRemoved,
        ControlBoundaryV2::IntentRetirementSynced,
    ];
    for (action, phases) in [
        (ReplacementRecoveryActionV2::Complete, complete.as_slice()),
        (ReplacementRecoveryActionV2::RetainOld, retain.as_slice()),
    ] {
        for &target in phases {
            let scratch = pending_intent("node-returned-recovery-error");
            let before_store = snapshot(&scratch.root());
            let mut reached = false;
            let result = recover_inner(
                &control_root(&scratch),
                &scratch.root(),
                independent_identity(),
                action,
                |boundary| {
                    if boundary == target {
                        reached = true;
                        Err(injected())
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(reached, "missing recovery hook {action:?}/{target:?}");
            assert!(matches!(result, Err(NodeControlErrorV2::Io(_))));
            let candidate = action == ReplacementRecoveryActionV2::Complete
                && target != ControlBoundaryV2::IntentResynchronized;
            if !candidate {
                assert_eq!(snapshot(&scratch.root()), before_store);
            }
            assert_released(&scratch, candidate);
            if candidate && control_root(&scratch).join(INTENT_NAME).exists() {
                let before = snapshot(&scratch.0);
                assert!(
                    NodeControlV2::recover(
                        control_root(&scratch),
                        scratch.root(),
                        independent_identity(),
                        ReplacementRecoveryActionV2::RetainOld
                    )
                    .is_err()
                );
                assert_eq!(snapshot(&scratch.0), before);
            }
            explicit_retry_or_clean_open(
                &scratch,
                action,
                action == ReplacementRecoveryActionV2::Complete,
            );
        }
    }
}

fn mutate_authority(control: &Path, case: usize) {
    match case {
        0 => {
            let path = control.join(CONFIRMED_NAME);
            let mut bytes = fs::read(&path).unwrap();
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
            fs::write(path, bytes).unwrap();
        }
        1 => fs::write(control.join("foreign"), b"preserve synthetic foreign bytes").unwrap(),
        2 => {
            // A different fully valid local intent still cannot replace the
            // exact authority retained by this already-running operation.
            let alternate = IntentRecordV2::new(
                independent_identity(),
                checkpoint(&main_chain(3), 0),
                main_chain(3),
                main_chain(4),
            )
            .unwrap()
            .encode()
            .unwrap();
            assert_ne!(fs::read(control.join(INTENT_NAME)).unwrap(), alternate);
            fs::write(control.join(INTENT_NAME), alternate).unwrap();
        }
        _ => unreachable!(),
    }
}

#[test]
fn changed_authority_after_intent_sync_is_rejected_before_store_publication() {
    let _serial = PROCESSES.lock().unwrap();
    for case in 0..3 {
        let (scratch, node) = prepared("node-adopt-authority-changed");
        let before_store = snapshot(&scratch.root());
        let mut after_mutation = None;
        let result = node.adopt_inner(received(&preferred_fork()), |boundary| {
            if boundary == ControlBoundaryV2::IntentDirectorySynced {
                mutate_authority(&control_root(&scratch), case);
                after_mutation = Some(snapshot(&scratch.0));
            }
            Ok(())
        });
        assert!(after_mutation.is_some());
        assert!(result.is_err());
        assert_eq!(snapshot(&scratch.root()), before_store);
        assert_eq!(snapshot(&scratch.0), after_mutation.unwrap());
        assert_released(&scratch, false);
    }
}

#[test]
fn changed_authority_after_recovery_resync_is_rejected_before_store_mutation() {
    let _serial = PROCESSES.lock().unwrap();
    for case in 0..3 {
        let scratch = pending_intent("node-recovery-authority-changed");
        let before_store = snapshot(&scratch.root());
        let mut after_mutation = None;
        let result = recover_inner(
            &control_root(&scratch),
            &scratch.root(),
            independent_identity(),
            ReplacementRecoveryActionV2::Complete,
            |boundary| {
                if boundary == ControlBoundaryV2::IntentResynchronized {
                    mutate_authority(&control_root(&scratch), case);
                    after_mutation = Some(snapshot(&scratch.0));
                }
                Ok(())
            },
        );
        assert!(after_mutation.is_some());
        assert!(result.is_err());
        assert_eq!(snapshot(&scratch.root()), before_store);
        assert_eq!(snapshot(&scratch.0), after_mutation.unwrap());
        assert_released(&scratch, false);
    }
}

#[test]
fn changed_confirmation_is_detected_before_retirement_and_final_ready_returns() {
    let _serial = PROCESSES.lock().unwrap();
    for target in [
        ControlBoundaryV2::ConfirmedDirectorySynced,
        ControlBoundaryV2::IntentRetirementSynced,
    ] {
        let (scratch, node) = prepared("node-final-confirmation-changed");
        let mut after_mutation = None;
        let result = node.adopt_inner(received(&preferred_fork()), |boundary| {
            if boundary == target {
                mutate_authority(&control_root(&scratch), 0);
                after_mutation = Some(snapshot(&scratch.0));
            }
            Ok(())
        });
        assert!(after_mutation.is_some());
        assert!(result.is_err());
        assert_eq!(snapshot(&scratch.0), after_mutation.unwrap());
        assert_eq!(
            control_root(&scratch).join(INTENT_NAME).exists(),
            target == ControlBoundaryV2::ConfirmedDirectorySynced
        );
        assert_released(&scratch, true);
    }

    let scratch = Scratch::new("node-initialization-final-record-changed");
    let old = main_chain(3);
    drop(ReplacementStoreV2::initialize(scratch.root(), &old).unwrap());
    let before_store = snapshot(&scratch.root());
    let mut after_mutation = None;
    let result = initialize_inner(
        &control_root(&scratch),
        &scratch.root(),
        independent_identity(),
        checkpoint(&old, 0),
        |boundary| {
            if boundary == ControlBoundaryV2::InitializedDirectorySynced {
                mutate_authority(&control_root(&scratch), 0);
                after_mutation = Some(snapshot(&scratch.0));
            }
            Ok(())
        },
    );
    assert!(after_mutation.is_some());
    assert!(result.is_err());
    assert_eq!(snapshot(&scratch.root()), before_store);
    assert_eq!(snapshot(&scratch.0), after_mutation.unwrap());
    assert_released(&scratch, false);
}

#[test]
fn foreign_confirmation_staging_after_store_success_is_never_overwritten_or_hidden() {
    let _serial = PROCESSES.lock().unwrap();
    for mismatch in [false, true] {
        let (scratch, node) = prepared("node-foreign-confirmation-prefix");
        let foreign = if mismatch {
            let mut bytes =
                ControlRecordV2::from_chain(independent_identity(), &preferred_fork(), 1)
                    .unwrap()
                    .encode()[..32]
                    .to_vec();
            bytes[20] ^= 1;
            bytes
        } else {
            b"foreign".to_vec()
        };
        let mut after_mutation = None;
        let result = node.adopt_inner(received(&preferred_fork()), |boundary| {
            if boundary == ControlBoundaryV2::StorePublicationCompleted {
                fs::write(
                    control_root(&scratch).join(CONFIRMED_PENDING_NAME),
                    &foreign,
                )
                .unwrap();
                after_mutation = Some(snapshot(&scratch.0));
            }
            Ok(())
        });
        assert!(after_mutation.is_some());
        assert!(result.is_err());
        assert_eq!(snapshot(&scratch.0), after_mutation.unwrap());
        assert_eq!(
            fs::read(control_root(&scratch).join(CONFIRMED_PENDING_NAME)).unwrap(),
            foreign
        );
        assert!(control_root(&scratch).join(INTENT_NAME).is_file());
        assert_released(&scratch, true);
        let before = snapshot(&scratch.0);
        assert!(
            NodeControlV2::recover(
                control_root(&scratch),
                scratch.root(),
                independent_identity(),
                ReplacementRecoveryActionV2::Complete
            )
            .is_err()
        );
        assert_eq!(snapshot(&scratch.0), before);
    }
}
