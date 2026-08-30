use mirror_consensus::{INITIAL_POW_BITS, mine, validate_pow};
use mirror_core::BlockHeader;
use mirror_crypto::Hash256;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mirror Node");
    println!("===========");
    println!();

    let mut header = BlockHeader::new(
        1,
        Hash256::default(),
        Hash256::default(),
        0,
        INITIAL_POW_BITS,
        0,
    );

    println!("Mining Mirror PoW header...");
    println!("Difficulty bits: {:#010x}", header.pow_bits());

    let result = mine(&mut header)?;

    println!();
    println!("Proof of Work found");
    println!("Nonce:    {}", result.nonce);
    println!("Attempts: {}", result.attempts);
    println!("Hash:     {}", result.hash);

    validate_pow(&header)?;

    println!("PoW:      VALID");

    Ok(())
}
