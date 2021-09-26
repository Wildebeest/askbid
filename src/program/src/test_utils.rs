use borsh::BorshSerialize;
use solana_sdk::rent::Rent;

pub fn space(account: &impl BorshSerialize) -> Result<usize, std::io::Error> {
    Ok(account.try_to_vec()?.len())
}

pub fn minimum_balance(account: &impl BorshSerialize) -> Result<u64, std::io::Error> {
    let space = space(account)?;
    Ok(Rent::default().minimum_balance(space))
}
