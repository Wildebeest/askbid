use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use thiserror::Error;
mod instructions;
use instructions::*;

#[cfg(test)]
mod test_utils;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SearchMarketError {}

pub const LAMPORTS_PER_TOKEN: u64 = 100_000;

solana_program::declare_id!("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let instruction = SearchMarketInstruction::try_from_slice(input)?;
    msg!("Instruction: {:?}", instruction);
    match instruction {
        SearchMarketInstruction::CreateMarket {
            expires_slot_offset,
            search_string,
        } => create_market(program_id, accounts, expires_slot_offset, search_string),
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
        SearchMarketInstruction::FillOrder {
            sol_escrow_bump_seed,
            token_escrow_bump_seed,
        } => fill_order(
            program_id,
            accounts,
            sol_escrow_bump_seed,
            token_escrow_bump_seed,
        ),
        SearchMarketInstruction::CancelOrder => cancel_order(program_id, accounts),
    }
}
