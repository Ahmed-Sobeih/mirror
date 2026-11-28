//! Finite public-synthetic V2 control-state restart demonstration.
//!
//! Each phase starts a different process. It uses only the production public
//! APIs; interruption boundaries and explicit crash recovery are tested by the
//! separate bounded subprocess harness, without a production fault-injection API.

#[path = "../tests/support_control_v2/mod.rs"]
mod support;

use std::{
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use mirror_chain::v2::{ChainV2, ForkChoiceV2};
use mirror_node::v2::{NodeControlV2, NodeStatusV2};
use mirror_storage::v2::ReplacementStoreV2;

use support::{
    Scratch, assert_complete_state, checkpoint,
    history::{fork_chain, independent_identity, main_chain},
    preferred_fork, received,
};

struct PhaseChild(Option<Child>);

impl PhaseChild {
    fn run(parent: &Path, phase: &str) -> Result<(), Box<dyn std::error::Error>> {
        let child = Command::new(std::env::current_exe()?)
            .arg("--synthetic-phase")
            .arg(phase)
            .arg(parent)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        assert_ne!(child.id(), std::process::id());
        let mut owner = Self(Some(child));
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = owner.0.as_mut().unwrap().try_wait()? {
                owner.0.take();
                if !status.success() {
                    return Err(format!("synthetic phase {phase} failed: {status}").into());
                }
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(
                    format!("synthetic phase {phase} exceeded its process deadline").into(),
                );
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for PhaseChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn show(label: &str, chain: &ChainV2, generation: u64) {
    let membership = chain.ledger().membership();
    println!(
        "{label}: height={} generation={generation} accounts={} active={} pending={} peak={} approvals={} MRB2={} state_items={}",
        chain.height(),
        chain.ledger().account_count(),
        membership.active_count(),
        membership.pending_count(),
        membership.active_high_water(),
        membership.retained_approval_count(),
        chain.encoded_bytes(),
        chain.state_items_used(),
    );
}

fn phase(name: &str, parent: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let control = parent.join("control");
    let store = parent.join("selected-store");
    let identity = independent_identity();
    let old = main_chain(3);
    match name {
        "initialize" => {
            assert_eq!(
                NodeControlV2::status(&control, &store, identity)?,
                NodeStatusV2::Missing
            );
            drop(ReplacementStoreV2::initialize(&store, &old)?);
            let node = NodeControlV2::initialize(&control, &store, identity, checkpoint(&old, 0))?;
            assert_complete_state(node.chain(), &old);
            show(
                "Initial independently pinned control state",
                node.chain(),
                node.checkpoint().generation,
            );
        }
        "keep-tie" => {
            let node = NodeControlV2::open_writer(&control, &store, identity)?;
            assert_eq!(node.checkpoint(), checkpoint(&old, 0));
            let fork = fork_chain();
            assert_eq!(
                old.ledger().account_bytes()?,
                fork.ledger().account_bytes()?
            );
            assert_ne!(old.ledger().membership(), fork.ledger().membership());
            let outcome = node.adopt(received(&fork))?;
            assert_eq!(outcome.report().choice, ForkChoiceV2::KeepCurrent);
            let node = outcome.into_control();
            assert_complete_state(node.chain(), &old);
            show(
                "Restart kept an equal-height membership fork",
                node.chain(),
                node.checkpoint().generation,
            );
        }
        "adopt" => {
            let node = NodeControlV2::open_writer(&control, &store, identity)?;
            let candidate = preferred_fork();
            assert_eq!(
                old.ledger().account_bytes()?,
                candidate.ledger().account_bytes()?
            );
            assert_ne!(old.ledger().membership(), candidate.ledger().membership());
            let outcome = node.adopt(received(&candidate))?;
            assert_eq!(outcome.report().choice, ForkChoiceV2::PreferCandidate);
            let node = outcome.into_control();
            assert_eq!(node.checkpoint(), checkpoint(&candidate, 1));
            assert_complete_state(node.chain(), &candidate);
            show(
                "Restart published and confirmed the preferred history",
                node.chain(),
                node.checkpoint().generation,
            );
        }
        "verify" => {
            let candidate = preferred_fork();
            assert_eq!(
                NodeControlV2::status(&control, &store, identity)?,
                NodeStatusV2::Clean {
                    checkpoint: checkpoint(&candidate, 1)
                },
            );
            let node = NodeControlV2::open_writer(&control, &store, identity)?;
            assert_complete_state(node.chain(), &candidate);
            show(
                "Fresh process reopened the complete confirmed history",
                node.chain(),
                node.checkpoint().generation,
            );
        }
        _ => return Err("unknown synthetic node-control phase".into()),
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    if arguments.next().as_deref() == Some(std::ffi::OsStr::new("--synthetic-phase")) {
        let name = arguments.next().ok_or("missing synthetic phase")?;
        let parent = arguments.next().ok_or("missing synthetic parent")?;
        return phase(
            name.to_str().ok_or("non-UTF8 synthetic phase")?,
            Path::new(&parent),
        );
    }
    let scratch = Scratch::new("node-control-demo");
    for name in ["initialize", "keep-tie", "adopt", "verify"] {
        PhaseChild::run(&scratch.0, name)?;
    }
    println!(
        "Four distinct process phases verified persisted local authority and complete membership-fork state."
    );
    println!(
        "The control directory is separate from the strict store. Only public synthetic histories were used."
    );
    Ok(())
}
