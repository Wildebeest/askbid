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
mod create_order;
pub use create_order::{create_order, create_order_instruction, OrderAccount, OrderSide};
mod fill_order;
pub use fill_order::{fill_order, fill_order_instruction};
mod cancel_order;
pub use cancel_order::{cancel_order, cancel_order_instruction};

#[cfg(test)]
pub mod test_utils {
    pub use super::create_market::test::*;
    pub use super::create_order::test::*;
    pub use super::create_result::test::*;
    pub use super::decide::test::*;
    pub use super::deposit::test::*;
    pub use super::withdraw::test::*;
}

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialEq)]
pub enum AccountType {
    SearchMarket,
    Result,
    Order,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum SearchMarketInstruction {
    CreateMarket {
        expires_slot_offset: u64,
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
    CreateOrder {
        side: u8,
        price: u64,
        quantity: u64,
        escrow_bump_seed: u8,
    },
    FillOrder {
        sol_escrow_bump_seed: u8,
        token_escrow_bump_seed: u8,
    },
    CancelOrder,
}
