use std::time::{SystemTime, UNIX_EPOCH};

use mirror_consensus::{INITIAL_POW_BITS, mine_block, validate_block};

use mirror_core::{
    Address, Block, MIRROR_CHAIN_ID, NUSA_PER_MRY, SignedTransaction, TRANSACTION_KIND_TRANSFER,
    TRANSACTION_VERSION, TransactionBody,
};

use mirror_crypto::{Hash256, Keypair};

use mirror_state::{ChainState, execute_transactions, validate_block_state};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node");
    println!("===========");
    println!();

    let alice = Keypair::generate()?;
    let bob = Keypair::generate()?;

    let alice_address = Address::from_public_key(&alice.public_key());

    let bob_address = Address::from_public_key(&bob.public_key());

    println!("Alice: {alice_address}");
    println!("Bob:   {bob_address}");
    println!();

    // Temporary pre-state used while the permanent Mirror genesis
    // specification is still being designed.
    //
    // This is NOT the final mainnet genesis allocation.
    let pre_state = ChainState::from_genesis_allocations([(alice_address, 100 * NUSA_PER_MRY)])?;

    println!("Pre-state");
    println!(
        "Alice balance: {} Nusa",
        pre_state.account(alice_address).balance()
    );
    println!(
        "Bob balance:   {} Nusa",
        pre_state.account(bob_address).balance()
    );
    println!("State root:    {}", pre_state.state_root());
    println!();

    let body = TransactionBody::new(
        TRANSACTION_VERSION,
        TRANSACTION_KIND_TRANSFER,
        MIRROR_CHAIN_ID,
        0,
        alice_address,
        bob_address,
        NUSA_PER_MRY,
        0,
        Vec::new(),
    );

    let transaction = SignedTransaction::sign(body, &alice)?;

    transaction.verify()?;

    println!("Created signed transfer");
    println!("Amount: 1 MRY = {} Nusa", NUSA_PER_MRY);
    println!("TXID:   {}", transaction.txid()?);
    println!();

    let transition = execute_transactions(&pre_state, std::slice::from_ref(&transaction))?;

    println!("Transaction executed");
    println!("Post-state root: {}", transition.state_root());
    println!();

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let mut block = Block::new(
        1,
        Hash256::default(),
        transition.state_root(),
        timestamp,
        INITIAL_POW_BITS,
        vec![transaction],
    )?;

    println!("Block assembled");
    println!("Transactions:     {}", block.transactions().len());
    println!("Transaction root: {}", block.header().transaction_root());
    println!("State root:       {}", block.header().state_root());
    println!();

    println!("Mining complete Mirror block...");

    let result = mine_block(&mut block)?;

    println!();
    println!("Block mined");
    println!("Nonce:    {}", result.nonce);
    println!("Attempts: {}", result.attempts);
    println!("Hash:     {}", result.hash);
    println!();

    validate_block(&block)?;

    let verified_transition = validate_block_state(&pre_state, &block)?;

    let post_state = verified_transition.post_state();

    println!("Block validation");
    println!("Signature:  VALID");
    println!("TX root:    VALID");
    println!("State root: VALID");
    println!("PoW:        VALID");
    println!();

    println!("Post-state");
    println!(
        "Alice balance: {} Nusa",
        post_state.account(alice_address).balance()
    );
    println!(
        "Alice nonce:   {}",
        post_state.account(alice_address).nonce()
    );
    println!(
        "Bob balance:   {} Nusa",
        post_state.account(bob_address).balance()
    );

    Ok(())
}
