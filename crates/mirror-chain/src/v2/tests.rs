use super::*;

#[test]
fn resource_caps_accept_exact_boundaries_and_reject_one_more() {
    let before = Resources {
        blocks: MAX_CHAIN_BLOCKS_V2 - 1,
        bytes: MAX_CHAIN_BYTES_V2 - 168,
        items: MAX_CHAIN_STATE_ITEMS_V2 - 1,
    };
    let exact = before.next(168, 1).unwrap();
    assert_eq!(exact.blocks, MAX_CHAIN_BLOCKS_V2);
    assert_eq!(exact.bytes, MAX_CHAIN_BYTES_V2);
    assert_eq!(exact.items, MAX_CHAIN_STATE_ITEMS_V2);
    assert_eq!(exact.next(0, 0), Err(ChainErrorV2::TooManyBlocks));
    assert_eq!(
        Resources { blocks: 0, ..exact }.next(1, 0),
        Err(ChainErrorV2::TooManyBytes)
    );
    assert_eq!(
        Resources {
            blocks: 0,
            bytes: 0,
            ..exact
        }
        .next(0, 1),
        Err(ChainErrorV2::StateItemLimit)
    );
    assert_eq!(before.items, MAX_CHAIN_STATE_ITEMS_V2 - 1);
}

#[test]
fn resource_arithmetic_never_wraps() {
    for (before, bytes, items) in [
        (
            Resources {
                blocks: usize::MAX,
                ..Resources::default()
            },
            0,
            0,
        ),
        (
            Resources {
                bytes: usize::MAX,
                ..Resources::default()
            },
            1,
            0,
        ),
        (
            Resources {
                items: u64::MAX,
                ..Resources::default()
            },
            0,
            1,
        ),
    ] {
        assert_eq!(before.next(bytes, items), Err(ChainErrorV2::LengthOverflow));
    }
}

#[test]
fn replay_aggregate_bytes_are_checked_without_overflow() {
    assert_eq!(
        add_replay_bytes(0, MAX_CHAIN_BYTES_V2),
        Ok(MAX_CHAIN_BYTES_V2)
    );
    assert_eq!(
        add_replay_bytes(MAX_CHAIN_BYTES_V2, 1),
        Err(ChainReplayErrorV2::Input(ChainErrorV2::TooManyBytes))
    );
    assert_eq!(
        add_replay_bytes(usize::MAX, 1),
        Err(ChainReplayErrorV2::Input(ChainErrorV2::LengthOverflow))
    );
}
