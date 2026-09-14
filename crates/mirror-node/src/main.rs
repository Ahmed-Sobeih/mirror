use std::time::{SystemTime, UNIX_EPOCH};

use mirror_chain::{Chain, GenesisConfig};

use mirror_consensus::INITIAL_POW_BITS;

use mirror_core::{
    Address, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody,
};

use mirror_crypto::Keypair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node");
    println!("===========");
    println!();

    // Temporary development wallets.
    //
    // Permanent wallet storage and the final mainnet genesis
    // configuration will be implemented later.
    let alice = Keypair::generate()?;
    let bob = Keypair::generate()?;

    let alice_address = Address::from_public_key(&alice.public_key());

    let bob_address = Address::from_public_key(&bob.public_key());

    println!("Alice: {alice_address}");
    println!("Bob:   {bob_address}");
    println!();

    let genesis_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // DEVELOPMENT GENESIS ONLY.
    //
    // Alice receives 100 MRY so we can exercise the real
    // account-state machinery.
    //
    // This is NOT Mirror's final mainnet allocation.
    let genesis = GenesisConfig::new(
        genesis_timestamp,
        INITIAL_POW_BITS,
        vec![(alice_address, 100 * NUSA_PER_MRY)],
    );

    println!("Creating Mirror development genesis...");

    let mut chain = Chain::from_genesis(genesis)?;

    let genesis_hash = chain.genesis().hash();

    println!();
    println!("Genesis accepted");
    println!("Height:     {}", chain.height());
    println!("Hash:       {genesis_hash}");
    println!("State root: {}", chain.state().state_root());
    println!(
        "Alice:      {} MRY",
        chain.state().account(alice_address).balance() / NUSA_PER_MRY
    );
    println!();

    // Alice sends Bob exactly 1 MRY.
    let alice_nonce = chain.state().account(alice_address).nonce();

    let body = TransactionBody::new(
        TRANSACTION_VERSION,
        TRANSACTION_KIND_TRANSFER,
        MIRROR_CHAIN_ID,
        alice_nonce,
        alice_address,
        bob_address,
        NUSA_PER_MRY,
        0,
        Vec::new(),
    );

    let transaction = SignedTransaction::sign(body, &alice)?;

    println!("Transaction created");
    println!("TXID:   {}", transaction.txid()?);
    println!("Amount: 1 MRY");
    println!("Nonce:  {alice_nonce}");
    println!();

    println!("Mining Block 1...");

    let block = chain.mine_next_block(vec![transaction], genesis_timestamp.saturating_add(1))?;

    println!();
    println!("Candidate Block 1 mined");
    println!("Previous: {}", block.header().previous_block_hash());
    println!("Hash:     {}", block.hash());
    println!("TX root:  {}", block.header().transaction_root());
    println!("State:    {}", block.header().state_root());
    println!("Nonce:    {}", block.header().nonce());
    println!();

    // The candidate goes through the exact same validation path
    // as a block received from another peer.
    chain.append_block(block)?;

    println!("Block 1 accepted");
    println!("================");
    println!("Chain height: {}", chain.height());
    println!("Chain blocks: {}", chain.len());
    println!("Tip hash:     {}", chain.tip_hash());
    println!();

    let alice_account = chain.state().account(alice_address);

    let bob_account = chain.state().account(bob_address);

    println!("Ledger state");
    println!("Alice: {} MRY", alice_account.balance() / NUSA_PER_MRY);
    println!("Alice nonce: {}", alice_account.nonce());
    println!("Bob:   {} MRY", bob_account.balance() / NUSA_PER_MRY);
    println!("Bob nonce:   {}", bob_account.nonce());
    println!();

    println!("Mirror chain: VALID");

    Ok(())
}
