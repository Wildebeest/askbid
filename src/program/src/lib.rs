use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

pub mod instruction {
    #[repr(C)]
    #[derive(Clone, Debug, PartialEq)]
    pub enum SearchMarketInstruction {
        Initialize,
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum SearchMarketState {
    Open,
    Closed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SearchMarketOrder {
    pub issuer: Pubkey,
    pub price: u64,
    pub quantity: u64,
    pub result: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SearchMarketAccount {
    pub issuer: Pubkey,
    pub search_string: String,
    pub order_book: Vec<SearchMarketOrder>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ResultAccount {
    pub url: String,
    pub title: String,
    pub description: String,
}

entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    _program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    _accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");

    Ok(())
}

