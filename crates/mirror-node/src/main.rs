use std::{
    env,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use mirror_chain::{Chain, GenesisConfig};

use mirror_consensus::INITIAL_POW_BITS;

use mirror_core::{
    Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody,
};

use mirror_crypto::Keypair;

use mirror_network::{PeerConnection, PeerListener};

use mirror_protocol::{HelloMessage, WireMessage};

use mirror_storage::BlockStore;

const BLOCK_DIRECTORY: &str = "mirror-data/devnet/blocks";

const DEV_GENESIS_TIMESTAMP: u64 = 1_800_000_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None | Some("mine") => run_miner(),

        Some("listen") => {
            let address = parse_address(args.next())?;

            run_listener(address)
        }

        Some("connect") => {
            let address = parse_address(args.next())?;

            run_connector(address)
        }

        Some(other) => Err(format!(
            "unknown command '{other}'\n\
                 usage:\n\
                 mirror-node mine\n\
                 mirror-node listen <IP:PORT>\n\
                 mirror-node connect <IP:PORT>"
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

/// Load the persisted development chain.
///
/// If no chain exists yet, create and persist genesis only.
fn load_chain() -> Result<(Chain, BlockStore, Keypair, Keypair), Box<dyn std::error::Error>> {
    let (alice, bob) = development_keys();

    let alice_address = Address::from_public_key(&alice.public_key());

    let genesis_config = development_genesis(alice_address);

    let store = BlockStore::open(BLOCK_DIRECTORY)?;

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

fn run_listener(address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — LISTEN");
    println!("====================");
    println!();

    let (chain, _store, _alice, _bob) = load_chain()?;

    println!("Local blockchain");
    print_local_chain(&chain);
    println!();

    let listener = PeerListener::bind(address)?;

    println!("Listening on {}", listener.local_addr()?);

    println!("Waiting for Mirror peer...");
    println!();

    let mut peer = listener.accept()?;

    println!("Peer connected from {}", peer.peer_addr()?);

    let remote = peer.receive_message()?;

    let remote_hello = match remote {
        WireMessage::Hello(hello) => hello,
    };

    validate_peer_hello(&chain, remote_hello)?;

    println!();
    println!("Received peer Hello");
    print_remote_hello(remote_hello);

    let local_hello = hello_for_chain(&chain);

    peer.send_message(&local_hello)?;

    println!();
    println!("Sent local Hello");

    println!("Handshake: VALID");

    Ok(())
}

fn run_connector(address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — CONNECT");
    println!("=====================");
    println!();

    let (chain, _store, _alice, _bob) = load_chain()?;

    println!("Local blockchain");
    print_local_chain(&chain);
    println!();

    println!("Connecting to {address}...");

    let mut peer = PeerConnection::connect(address)?;

    println!("Connected to {}", peer.peer_addr()?);

    peer.send_message(&hello_for_chain(&chain))?;

    println!("Sent local Hello");

    let remote = peer.receive_message()?;

    let remote_hello = match remote {
        WireMessage::Hello(hello) => hello,
    };

    validate_peer_hello(&chain, remote_hello)?;

    println!();
    println!("Received peer Hello");
    print_remote_hello(remote_hello);

    println!();
    println!("Handshake: VALID");

    Ok(())
}

fn run_miner() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node — MINE");
    println!("==================");
    println!();

    let (mut chain, store, alice, bob) = load_chain()?;

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

    let mut validated_chain = chain.clone();

    validated_chain.append_block(block.clone())?;

    store.write_block(next_height, &block)?;

    chain = validated_chain;

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
