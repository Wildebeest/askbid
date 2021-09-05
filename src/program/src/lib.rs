use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Slot,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use thiserror::Error;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum SearchMarketInstruction {
    CreateMarket {
        expires_slot: Slot,
        search_string: String,
    },
    CreateResult {
        search_market: Pubkey,
        url: String,
        name: String,
        snippet: String,
    },
    Deposit,
    Withdraw,
    Decide,
}

pub fn create_market_instruction(
    program_id: &Pubkey,
    market_pubkey: &Pubkey,
    expires_slot: Slot,
    search_string: String,
) -> Result<Instruction, std::io::Error> {
    let data = SearchMarketInstruction::CreateMarket {
        expires_slot,
        search_string,
    }
    .try_to_vec()?;
    let accounts = vec![AccountMeta::new(*market_pubkey, false)];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct SearchMarketAccount {
    pub search_string: String,
    pub best_result: Option<Pubkey>,
    pub expires_slot: Slot,
}

impl SearchMarketAccount {
    pub fn space(&self) -> Result<usize, std::io::Error> {
        Ok(self.try_to_vec()?.len())
    }
    pub fn minimum_balance(&self) -> Result<u64, std::io::Error> {
        let space = self.space()?;
        Ok(Rent::default().minimum_balance(space))
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ResultAccount {
    pub search_market: Pubkey,
    pub url: String,
    pub name: String,
    pub snippet: String,
    pub yes_mint: Pubkey,
    pub no_mint: Pubkey,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SearchMarketError {}

pub fn create_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    expires_slot: Slot,
    search_string: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let market_account_info = next_account_info(account_info_iter)?;
    let search_market = SearchMarketAccount {
        best_result: None,
        search_string,
        expires_slot,
    };

    search_market
        .serialize(&mut *market_account_info.data.borrow_mut())
        .map_err(|e| e.into())
}

pub fn create_result(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    search_market: Pubkey,
    url: String,
    name: String,
    snippet: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            yes_mint_account_info.key,
            program_id,
            None,
            spl_token::native_mint::DECIMALS,
        )?,
        &[yes_mint_account_info.clone(), rent_account_info.clone()],
    )?;

    let no_mint_account_info = next_account_info(account_info_iter)?;
    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            no_mint_account_info.key,
            program_id,
            None,
            spl_token::native_mint::DECIMALS,
        )?,
        &[no_mint_account_info.clone(), rent_account_info.clone()],
    )?;

    let result = ResultAccount {
        search_market,
        url,
        name,
        snippet,
        yes_mint: *yes_mint_account_info.key,
        no_mint: *no_mint_account_info.key,
    };

    result
        .serialize(&mut *result_account_info.data.borrow_mut())
        .map_err(|e| e.into())
}

solana_program::declare_id!("My11111111111111111111111111111111111111111");
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    input: &[u8],        // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    let instruction = SearchMarketInstruction::try_from_slice(input)?;
    match instruction {
        SearchMarketInstruction::CreateMarket {
            expires_slot,
            search_string,
        } => create_market(program_id, accounts, expires_slot, search_string),
        SearchMarketInstruction::CreateResult {
            search_market,
            url,
            name,
            snippet,
        } => create_result(program_id, accounts, search_market, url, name, snippet),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_sdk::account::{
        Account as SolanaAccount,
    };

    #[test]
    pub fn test_create_market() {
        let program_id = crate::id();
        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let market_key = Pubkey::new_unique();
        let mut market_account = SolanaAccount::new(
            market.minimum_balance().unwrap(),
            market.space().unwrap(),
            &program_id,
        );
        let create_market_instruction = create_market_instruction(
            &program_id,
            &market_key,
            market.expires_slot,
            market.search_string.clone(),
        )
        .unwrap();

        let accounts = create_market_instruction
            .accounts
            .iter()
            .zip(vec![&mut market_account])
            .map(|(account_meta, account)| {
                (&account_meta.pubkey, account_meta.is_signer, account).into()
            })
            .collect::<Vec<_>>();

        process_instruction(&program_id, &accounts[..], &create_market_instruction.data).unwrap();

        let processed_market =
            SearchMarketAccount::try_from_slice(&market_account.data[..]).unwrap();
        assert_eq!(market, processed_market);
    }

    #[test]
    pub fn test_create_result() {

    }
}
