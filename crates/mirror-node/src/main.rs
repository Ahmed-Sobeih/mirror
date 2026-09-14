use std::time::{SystemTime, UNIX_EPOCH};

use mirror_chain::{Chain, GenesisConfig};

use mirror_consensus::INITIAL_POW_BITS;

use mirror_core::{
    Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody,
};

use mirror_crypto::Keypair;

use mirror_storage::BlockStore;

/// Development-only blockchain directory.
///
/// These files are intentionally outside Git.
const BLOCK_DIRECTORY: &str = "mirror-data/devnet/blocks";

/// Fixed development genesis timestamp.
///
/// Genesis must be identical after every node restart.
const DEV_GENESIS_TIMESTAMP: u64 = 1_800_000_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node");
    println!("===========");
    println!();

    // DEVELOPMENT KEYS ONLY.
    //
    // Fixed keys are necessary for reproducible development genesis
    // until persistent encrypted wallet storage is implemented.
    //
    // These keys must NEVER be used for mainnet funds.
    let alice = Keypair::from_secret_bytes([1u8; 32]);

    let bob = Keypair::from_secret_bytes([2u8; 32]);

    let alice_address = Address::from_public_key(&alice.public_key());

    let bob_address = Address::from_public_key(&bob.public_key());

    println!("Development accounts");
    println!("Alice: {alice_address}");
    println!("Bob:   {bob_address}");
    println!();

    let genesis_config = GenesisConfig::new(
        DEV_GENESIS_TIMESTAMP,
        INITIAL_POW_BITS,
        vec![(alice_address, 100 * NUSA_PER_MRY)],
    );

    let store = BlockStore::open(BLOCK_DIRECTORY)?;

    let stored_blocks = store.read_contiguous_blocks()?;

    let mut chain = if stored_blocks.is_empty() {
        println!("No persisted blockchain found.");

        println!("Creating development genesis...");

        let chain = Chain::from_genesis(genesis_config.clone())?;

        store.write_block(0, chain.genesis())?;

        println!("Genesis persisted to disk.");
        println!();

        chain
    } else {
        println!("Found {} persisted block(s).", stored_blocks.len());

        println!("Revalidating blockchain from genesis...");

        let chain = Chain::from_persisted_blocks(genesis_config.clone(), stored_blocks)?;

        println!("Blockchain recovered successfully.");
        println!();

        chain
    };

    println!("Recovered chain");
    println!("Height:     {}", chain.height());
    println!("Blocks:     {}", chain.len());
    println!("Tip hash:   {}", chain.tip_hash());
    println!("State root: {}", chain.state().state_root());
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

    // Each invocation creates one new transfer using the nonce
    // reconstructed from persisted chain state.
    let transaction_body = TransactionBody::new(
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

    let transaction = SignedTransaction::sign(transaction_body, &alice)?;

    let txid = transaction.txid()?;

    println!("Creating next block");
    println!("TXID:   {txid}");
    println!("Amount: 1 MRY");
    println!("Nonce:  {}", alice_account.nonce());
    println!();

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // Keep development block timestamps monotonic even if the
    // machine clock moves backwards.
    let minimum_timestamp = chain.tip().header().timestamp().saturating_add(1);

    let timestamp = now.max(minimum_timestamp);

    let block = chain.mine_next_block(vec![transaction], timestamp)?;

    let next_height = chain
        .height()
        .checked_add(1)
        .ok_or_else(|| std::io::Error::other("chain height overflow"))?;

    println!("Block {next_height} mined");
    println!("Previous: {}", block.header().previous_block_hash());
    println!("Hash:     {}", block.hash());
    println!("State:    {}", block.header().state_root());
    println!("PoW nonce: {}", block.header().nonce());
    println!();

    // Validate using a cloned chain first.
    //
    // Only after complete validation do we publish the canonical
    // block bytes to disk. Once persistence succeeds, the in-memory
    // chain advances to the same state.
    let mut validated_chain = chain.clone();

    validated_chain.append_block(block.clone())?;

    store.write_block(next_height, &block)?;

    chain = validated_chain;

    println!("Block {next_height} validated and persisted.");
    println!();

    println!("Updated chain");
    println!("Height:   {}", chain.height());
    println!("Blocks:   {}", chain.len());
    println!("Tip hash: {}", chain.tip_hash());
    println!();

    println!("Updated ledger");
    println!(
        "Alice: {} MRY",
        chain.state().account(alice_address).balance() / NUSA_PER_MRY
    );
    println!(
        "Alice nonce: {}",
        chain.state().account(alice_address).nonce()
    );
    println!(
        "Bob:   {} MRY",
        chain.state().account(bob_address).balance() / NUSA_PER_MRY
    );
    println!();

    println!("Mirror chain: VALID + PERSISTED");

    Ok(())
}
