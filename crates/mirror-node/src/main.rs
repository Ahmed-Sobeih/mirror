use std::{
    env,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_chain::{Chain, GenesisConfig};

use mirror_consensus::INITIAL_POW_BITS;

use mirror_core::{
    Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody, decode_block, encode_block,
};

use mirror_crypto::Keypair;

use mirror_network::{PeerConnection, PeerListener};

use mirror_protocol::{BlockDataMessage, GetBlockMessage, HelloMessage, WireMessage};

use mirror_storage::BlockStore;

const DEFAULT_BLOCK_DIRECTORY: &str = "mirror-data/devnet/blocks";

const DEV_GENESIS_TIMESTAMP: u64 = 1_800_000_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None | Some("mine") => {
            let directory = args
                .next()
                .unwrap_or_else(|| DEFAULT_BLOCK_DIRECTORY.to_string());

            run_miner(&directory)
        }

        Some("listen") => {
            let address = parse_address(args.next())?;

            let directory = args
                .next()
                .unwrap_or_else(|| DEFAULT_BLOCK_DIRECTORY.to_string());

            run_listener(address, &directory)
        }

        Some("connect") => {
            let address = parse_address(args.next())?;

            let directory = args
                .next()
                .unwrap_or_else(|| DEFAULT_BLOCK_DIRECTORY.to_string());

            run_connector(address, &directory)
        }

        Some(other) => Err(format!(
            "unknown command '{other}'\n\
                 usage:\n\
                 mirror-node mine [BLOCK_DIRECTORY]\n\
                 mirror-node listen <IP:PORT> [BLOCK_DIRECTORY]\n\
                 mirror-node connect <IP:PORT> [BLOCK_DIRECTORY]"
        )
        .into()),
    }
}

fn parse_address(value: Option<String>) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let value = value.ok_or("missing socket address, expected IP:PORT")?;

    Ok(value.parse()?)
}

fn development_keys() -> (Keypair, Keypair) {
    (
        Keypair::from_secret_bytes([1u8; 32]),
        Keypair::from_secret_bytes([2u8; 32]),
    )
}

fn development_genesis(alice_address: Address) -> GenesisConfig {
    GenesisConfig::new(
        DEV_GENESIS_TIMESTAMP,
        INITIAL_POW_BITS,
        vec![(alice_address, 100 * NUSA_PER_MRY)],
    )
}

fn load_chain(
    block_directory: &str,
) -> Result<(Chain, BlockStore, Keypair, Keypair), Box<dyn std::error::Error>> {
    let (alice, bob) = development_keys();

    let alice_address = Address::from_public_key(&alice.public_key());

    let genesis_config = development_genesis(alice_address);

    let store = BlockStore::open(block_directory)?;

    let stored_blocks = store.read_contiguous_blocks()?;

    let chain = if stored_blocks.is_empty() {
        let chain = Chain::from_genesis(genesis_config)?;

        store.write_block(0, chain.genesis())?;

        chain
    } else {
        Chain::from_persisted_blocks(genesis_config, stored_blocks)?
    };

    Ok((chain, store, alice, bob))
}

fn hello_for_chain(chain: &Chain) -> WireMessage {
    WireMessage::Hello(HelloMessage::new(
        MIRROR_CHAIN_ID,
        chain.genesis().hash(),
        chain.height(),
        chain.tip_hash(),
    ))
}

fn validate_peer_hello(
    chain: &Chain,
    hello: HelloMessage,
) -> Result<(), Box<dyn std::error::Error>> {
    if hello.chain_id() != MIRROR_CHAIN_ID {
        return Err(format!(
            "peer is on wrong chain id: expected {}, got {}",
            MIRROR_CHAIN_ID,
            hello.chain_id()
        )
        .into());
    }

    let expected_genesis = chain.genesis().hash();

    if hello.genesis_hash() != expected_genesis {
        return Err(format!(
            "peer genesis mismatch: expected {}, got {}",
            expected_genesis,
            hello.genesis_hash()
        )
        .into());
    }

    Ok(())
}

fn receive_hello(peer: &mut PeerConnection) -> Result<HelloMessage, Box<dyn std::error::Error>> {
    match peer.receive_message()? {
        WireMessage::Hello(hello) => Ok(hello),

        other => Err(format!("expected Hello message, got {other:?}").into()),
    }
}

fn print_local_chain(chain: &Chain) {
    println!("Genesis: {}", chain.genesis().hash());

    println!("Height:  {}", chain.height());

    println!("Tip:     {}", chain.tip_hash());
}

fn print_remote_hello(hello: HelloMessage) {
    println!("Peer chain ID: {}", hello.chain_id());

    println!("Peer genesis:  {}", hello.genesis_hash());

    println!("Peer height:   {}", hello.best_height());

    println!("Peer tip:      {}", hello.tip_hash());
}

fn run_listener(
    address: SocketAddr,
    block_directory: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — LISTEN");

    println!("====================");

    println!("Data: {block_directory}");

    println!();

    let (chain, store, _alice, _bob) = load_chain(block_directory)?;

    println!("Local blockchain");

    print_local_chain(&chain);

    println!();

    let listener = PeerListener::bind(address)?;

    println!("Listening on {}", listener.local_addr()?);

    println!("Waiting for Mirror peer...");

    println!();

    let mut peer = listener.accept()?;

    println!("Peer connected from {}", peer.peer_addr()?);

    let remote_hello = receive_hello(&mut peer)?;

    validate_peer_hello(&chain, remote_hello)?;

    println!();
    println!("Received peer Hello");

    print_remote_hello(remote_hello);

    peer.send_message(&hello_for_chain(&chain))?;

    println!();
    println!("Sent local Hello");

    println!("Handshake: VALID");

    if remote_hello.best_height() >= chain.height() {
        println!();

        if remote_hello.best_height() == chain.height()
            && remote_hello.tip_hash() != chain.tip_hash()
        {
            return Err(
                "peer has a different tip at the same height; fork handling is not implemented yet"
                    .into(),
            );
        }

        println!("No blocks to serve.");

        return Ok(());
    }

    let blocks_to_serve = chain.height() - remote_hello.best_height();

    println!();

    println!("Peer is behind by {blocks_to_serve} block(s).");

    for _ in 0..blocks_to_serve {
        let request = match peer.receive_message()? {
            WireMessage::GetBlock(request) => request,

            other => {
                return Err(format!("expected GetBlock, got {other:?}").into());
            }
        };

        let height = request.height();

        if height == 0 || height > chain.height() {
            return Err(format!("peer requested invalid block height {height}").into());
        }

        let block = store.read_block(height)?;

        let bytes = encode_block(&block)?;

        peer.send_message(&WireMessage::BlockData(BlockDataMessage::new(
            height, bytes,
        )))?;

        println!("Served block {height}");
    }

    println!();

    println!("Sync service complete.");

    Ok(())
}

fn run_connector(
    address: SocketAddr,
    block_directory: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — CONNECT");

    println!("=====================");

    println!("Data: {block_directory}");

    println!();

    let (mut chain, store, _alice, _bob) = load_chain(block_directory)?;

    println!("Local blockchain");

    print_local_chain(&chain);

    println!();

    println!("Connecting to {address}...");

    let mut peer = PeerConnection::connect(address)?;

    println!("Connected to {}", peer.peer_addr()?);

    peer.send_message(&hello_for_chain(&chain))?;

    println!("Sent local Hello");

    let remote_hello = receive_hello(&mut peer)?;

    validate_peer_hello(&chain, remote_hello)?;

    println!();

    println!("Received peer Hello");

    print_remote_hello(remote_hello);

    println!();

    println!("Handshake: VALID");

    if remote_hello.best_height() == chain.height() {
        if remote_hello.tip_hash() != chain.tip_hash() {
            return Err(
                "peer has a different tip at the same height; fork handling is not implemented yet"
                    .into(),
            );
        }

        println!("Already synchronized.");

        return Ok(());
    }

    if remote_hello.best_height() < chain.height() {
        println!("Local chain is ahead of peer.");

        println!("Reverse-direction sync is not implemented in connect mode yet.");

        return Ok(());
    }

    println!();

    println!(
        "Synchronizing {} missing block(s)...",
        remote_hello.best_height() - chain.height()
    );

    while chain.height() < remote_hello.best_height() {
        let next_height = chain
            .height()
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("chain height overflow"))?;

        peer.send_message(&WireMessage::GetBlock(GetBlockMessage::new(next_height)))?;

        println!("Requested block {next_height}");

        let block_data = match peer.receive_message()? {
            WireMessage::BlockData(block) => block,

            other => {
                return Err(format!("expected BlockData, got {other:?}").into());
            }
        };

        if block_data.height() != next_height {
            return Err(format!(
                "peer returned wrong block height: requested {next_height}, got {}",
                block_data.height()
            )
            .into());
        }

        let block = decode_block(block_data.block_bytes())?;

        let mut candidate = chain.clone();

        candidate.append_block(block.clone())?;

        store.write_block(next_height, &block)?;

        chain = candidate;

        println!("Validated and stored block {next_height}");
    }

    if chain.tip_hash() != remote_hello.tip_hash() {
        return Err(format!(
            "sync completed at advertised height but tip differs: local {}, peer {}",
            chain.tip_hash(),
            remote_hello.tip_hash()
        )
        .into());
    }

    println!();

    println!("Synchronization complete.");

    println!("Height: {}", chain.height());

    println!("Tip:    {}", chain.tip_hash());

    println!("Mirror chain: SYNCED + VALID");

    Ok(())
}

fn run_miner(block_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — MINE");

    println!("==================");

    println!("Data: {block_directory}");

    println!();

    let (mut chain, store, alice, bob) = load_chain(block_directory)?;

    let alice_address = Address::from_public_key(&alice.public_key());

    let bob_address = Address::from_public_key(&bob.public_key());

    println!("Recovered chain");

    print_local_chain(&chain);

    println!();

    let alice_account = chain.state().account(alice_address);

    let bob_account = chain.state().account(bob_address);

    println!("Current ledger");

    println!("Alice: {} MRY", alice_account.balance() / NUSA_PER_MRY);

    println!("Alice nonce: {}", alice_account.nonce());

    println!("Bob:   {} MRY", bob_account.balance() / NUSA_PER_MRY);

    println!();

    if alice_account.balance() < NUSA_PER_MRY {
        println!("Alice has no full MRY left to transfer.");

        return Ok(());
    }

    let body = TransactionBody::new(
        TRANSACTION_VERSION,
        TRANSACTION_KIND_TRANSFER,
        MIRROR_CHAIN_ID,
        alice_account.nonce(),
        alice_address,
        bob_address,
        NUSA_PER_MRY,
        0,
        Vec::new(),
    );

    let transaction = SignedTransaction::sign(body, &alice)?;

    println!("Creating block with TXID {}", transaction.txid()?);

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let minimum_timestamp = chain.tip().header().timestamp().saturating_add(1);

    let timestamp = now.max(minimum_timestamp);

    let block = chain.mine_next_block(vec![transaction], timestamp)?;

    let next_height = chain
        .height()
        .checked_add(1)
        .ok_or_else(|| std::io::Error::other("chain height overflow"))?;

    let mut candidate = chain.clone();

    candidate.append_block(block.clone())?;

    store.write_block(next_height, &block)?;

    chain = candidate;

    println!("Block {next_height} accepted");

    println!("Hash: {}", chain.tip_hash());

    println!(
        "Alice: {} MRY",
        chain.state().account(alice_address).balance() / NUSA_PER_MRY
    );

    println!(
        "Bob:   {} MRY",
        chain.state().account(bob_address).balance() / NUSA_PER_MRY
    );

    println!("Mirror chain: VALID + PERSISTED");

    Ok(())
}
