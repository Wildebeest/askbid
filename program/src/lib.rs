use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use thiserror::Error;
mod instructions;
use instructions::*;
mod undecided_result;

#[cfg(test)]
mod test_utils;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SearchMarketError {}

solana_program::declare_id!("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    input: &[u8],        // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    let instruction = SearchMarketInstruction::try_from_slice(input)?;
    msg!("Instruction: {:?}", instruction);
    match instruction {
        SearchMarketInstruction::CreateMarket {
            expires_slot,
            search_string,
        } => create_market(program_id, accounts, expires_slot, search_string),
        SearchMarketInstruction::CreateResult {
            url,
            name,
            snippet,
            bump_seed,
        } => create_result(program_id, accounts, url, name, snippet, bump_seed),
        SearchMarketInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        SearchMarketInstruction::Withdraw { amount } => withdraw(program_id, accounts, amount),
        SearchMarketInstruction::Decide => decide(program_id, accounts),
        SearchMarketInstruction::CreateOrder {
            side,
            price,
            quantity,
            escrow_bump_seed,
        } => create_order(
            program_id,
            accounts,
            side,
            price,
            quantity,
            escrow_bump_seed,
        ),
        _ => Ok(()),
    }
}
