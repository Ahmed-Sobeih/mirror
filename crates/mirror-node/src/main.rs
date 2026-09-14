use std::time::{SystemTime, UNIX_EPOCH};

use mirror_consensus::{INITIAL_POW_BITS, mine_block, validate_block};

use mirror_core::{
    Address, Block, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody,
};

use mirror_crypto::{Hash256, Keypair};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node");
    println!("===========");
    println!();

    // Temporary wallets generated for this run.
    // Persistent encrypted wallet storage comes later.
    let alice = Keypair::generate()?;
    let bob = Keypair::generate()?;

    let alice_address = Address::from_public_key(&alice.public_key());

    let bob_address = Address::from_public_key(&bob.public_key());

    println!("Alice: {alice_address}");
    println!("Bob:   {bob_address}");
    println!();

    // Create a cryptographically valid transfer.
    //
    // Economic/state validation is NOT implemented yet,
    // so Alice's balance is not checked at this stage.
    let body = TransactionBody::new(
        TRANSACTION_VERSION,
        TRANSACTION_KIND_TRANSFER,
        1,
        0,
        alice_address,
        bob_address,
        NUSA_PER_MRY,
        1_000,
        Vec::new(),
    );

    let transaction = SignedTransaction::sign(body, &alice)?;

    transaction.verify()?;

    let txid = transaction.txid()?;

    println!("Created signed MRY transaction");
    println!("TXID: {txid}");
    println!();

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // State root remains zero until we implement
    // the account/state-transition engine.
    let mut block = Block::new(
        1,
        Hash256::default(),
        Hash256::default(),
        timestamp,
        INITIAL_POW_BITS,
        vec![transaction],
    )?;

    println!("Block assembled");
    println!("Transactions:     {}", block.transactions().len());
    println!("Transaction root: {}", block.header().transaction_root());
    println!("State root:       {}", block.header().state_root());
    println!("Difficulty bits:  {:#010x}", block.header().pow_bits());
    println!();

    println!("Mining complete Mirror block...");

    let result = mine_block(&mut block)?;

    println!();
    println!("Block mined");
    println!("Nonce:    {}", result.nonce);
    println!("Attempts: {}", result.attempts);
    println!("Hash:     {}", result.hash);

    validate_block(&block)?;

    println!("Block:    VALID");
    println!("PoW:      VALID");
    println!("TX root:  VALID");

    Ok(())
}
