use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::clock::Slot;

mod create_market;
pub use create_market::{create_market, create_market_instruction, SearchMarketAccount};
mod create_result;
pub use create_result::{create_result, create_result_instruction, ResultAccount};
mod deposit;
pub use deposit::{deposit, deposit_instruction};
mod withdraw;
pub use withdraw::{withdraw, withdraw_instruction};
mod decide;
pub use decide::{decide, decide_instruction};

#[cfg(test)]
pub mod test_utils {
    pub use super::create_market::test::*;
    pub use super::create_result::test::*;
    pub use super::decide::test::*;
    pub use super::deposit::test::*;
    pub use super::withdraw::test::*;
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum SearchMarketInstruction {
    CreateMarket {
        expires_slot: Slot,
        search_string: String,
    },
    CreateResult {
        url: String,
        name: String,
        snippet: String,
        bump_seed: u8,
    },
    Deposit {
        amount: u64,
    },
    Withdraw {
        amount: u64,
    },
    Decide,
}
