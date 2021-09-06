use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Slot,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::transfer,
    sysvar,
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
        url: String,
        name: String,
        snippet: String,
    },
    Deposit {
        amount: u64,
    },
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

pub fn create_result_instruction(
    program_id: &Pubkey,
    result_pubkey: &Pubkey,
    market_pubkey: &Pubkey,
    yes_mint_pubkey: &Pubkey,
    no_mint_pubkey: &Pubkey,
    url: String,
    name: String,
    snippet: String,
) -> Result<Instruction, std::io::Error> {
    let data = SearchMarketInstruction::CreateResult { url, name, snippet }.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*result_pubkey, false),
        AccountMeta::new_readonly(*market_pubkey, false),
        AccountMeta::new(*yes_mint_pubkey, false),
        AccountMeta::new(*no_mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn deposit_instruction(
    program_id: &Pubkey,
    result_pubkey: &Pubkey,
    deposit_pubkey: &Pubkey,
    yes_mint_pubkey: &Pubkey,
    yes_token_pubkey: &Pubkey,
    no_mint_pubkey: &Pubkey,
    no_token_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction, std::io::Error> {
    let data = SearchMarketInstruction::Deposit { amount }.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*result_pubkey, false),
        AccountMeta::new(*deposit_pubkey, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*yes_mint_pubkey, false),
        AccountMeta::new(*yes_token_pubkey, false),
        AccountMeta::new(*no_mint_pubkey, false),
        AccountMeta::new(*no_token_pubkey, false),
    ];

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

pub fn space(account: &impl BorshSerialize) -> Result<usize, std::io::Error> {
    Ok(account.try_to_vec()?.len())
}
pub fn minimum_balance(account: &impl BorshSerialize) -> Result<u64, std::io::Error> {
    let space = space(account)?;
    Ok(Rent::default().minimum_balance(space))
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
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
    url: String,
    name: String,
    snippet: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let market_account_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;

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
        search_market: *market_account_info.key,
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

pub fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let deposit_account_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let yes_token_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let no_token_account_info = next_account_info(account_info_iter)?;

    let result = ResultAccount::try_from_slice(&result_account_info.data.borrow())?;

    invoke(
        &transfer(deposit_account_info.key, result_account_info.key, amount),
        &[deposit_account_info.clone(), result_account_info.clone()],
    )?;
    msg!("made it");

    invoke(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            &result.yes_mint,
            yes_token_account_info.key,
            yes_token_account_info.key,
            &[program_id],
            amount,
        )?,
        &[
            yes_mint_account_info.clone(),
            yes_token_account_info.clone(),
        ],
    )?;

    invoke(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            &result.no_mint,
            no_token_account_info.key,
            no_token_account_info.key,
            &[program_id],
            amount,
        )?,
        &[no_mint_account_info.clone(), no_token_account_info.clone()],
    )?;

    Ok(())
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
        SearchMarketInstruction::CreateResult { url, name, snippet } => {
            create_result(program_id, accounts, url, name, snippet)
        }
        SearchMarketInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, BanksClient, ProgramTest};
    use solana_sdk::{
        account::{create_account_for_test, Account as SolanaAccount},
        genesis_config::create_genesis_config,
        native_token::sol_to_lamports,
        signature::Signer,
        transaction::Transaction,
        system_program,
    };
    use spl_token::state::Mint;

    fn get_account_infos<'a>(
        instruction: &'a Instruction,
        accounts: Vec<&'a mut SolanaAccount>,
    ) -> Vec<AccountInfo<'a>> {
        instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| {
                (&account_meta.pubkey, account_meta.is_signer, account).into()
            })
            .collect::<Vec<_>>()
    }

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
            minimum_balance(&market).unwrap(),
            space(&market).unwrap(),
            &program_id,
        );
        let create_market_instruction = create_market_instruction(
            &program_id,
            &market_key,
            market.expires_slot,
            market.search_string.clone(),
        )
        .unwrap();

        let accounts = get_account_infos(&create_market_instruction, vec![&mut market_account]);
        process_instruction(&program_id, &accounts[..], &create_market_instruction.data).unwrap();

        let processed_market =
            SearchMarketAccount::try_from_slice(&market_account.data[..]).unwrap();
        assert_eq!(market, processed_market);
    }

    #[test]
    pub fn test_create_result() {
        let program_id = crate::id();
        let market_key = Pubkey::new_unique();
        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let mut market_account = SolanaAccount::new(
            minimum_balance(&market).unwrap(),
            space(&market).unwrap(),
            &program_id,
        );

        let result_key = Pubkey::new_unique();
        let result = ResultAccount {
            search_market: market_key,
            url: String::from("http://cyberpunk.net"),
            name: String::from("Cyberpunk website"),
            snippet: String::from("A game fated to be legend"),
            yes_mint: Pubkey::new_unique(),
            no_mint: Pubkey::new_unique(),
        };
        let mut result_account = SolanaAccount::new(
            minimum_balance(&result).unwrap(),
            space(&result).unwrap(),
            &program_id,
        );

        let mut yes_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &program_id,
        );

        let mut no_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &program_id,
        );

        let mut rent_account = create_account_for_test(&Rent::default());

        let create_result_instruction = create_result_instruction(
            &program_id,
            &result_key,
            &market_key,
            &result.yes_mint,
            &result.no_mint,
            result.url.clone(),
            result.name.clone(),
            result.snippet.clone(),
        )
        .unwrap();
        let accounts = get_account_infos(
            &create_result_instruction,
            vec![
                &mut result_account,
                &mut market_account,
                &mut yes_mint_account,
                &mut no_mint_account,
                &mut rent_account,
            ],
        );
        process_instruction(&program_id, &accounts[..], &create_result_instruction.data).unwrap();
        let processed_result = ResultAccount::try_from_slice(&result_account.data[..]).unwrap();
        assert_eq!(result, processed_result);
    }

    #[tokio::test]
    async fn test_deposit() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let result_pubkey = Pubkey::new_unique();
        let result = ResultAccount {
            search_market: Pubkey::new_unique(),
            url: String::from("http://cyberpunk.net"),
            name: String::from("Cyberpunk website"),
            snippet: String::from("A game fated to be legend"),
            yes_mint: Pubkey::new_unique(),
            no_mint: Pubkey::new_unique(),
        };
        let mut result_account = SolanaAccount::new(
            minimum_balance(&result).unwrap(),
            result.try_to_vec().unwrap().len(),
            &program_id,
        );
        result_account.data = result.try_to_vec().unwrap();
        program_test.add_account(result_pubkey, result_account);

        let deposit_pubkey = Pubkey::new_unique();
        let deposit_account = SolanaAccount::new(100, 0, &program_id);
        program_test.add_account(deposit_pubkey, deposit_account);

        let yes_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &program_id,
        );
        program_test.add_account(result.yes_mint, yes_mint_account);

        let no_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &program_id,
        );
        program_test.add_account(result.no_mint, no_mint_account);

        let yes_token_pubkey = Pubkey::new_unique();
        let yes_token_account = SolanaAccount::new(
            Rent::default().minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        program_test.add_account(yes_token_pubkey, yes_token_account);

        let no_token_pubkey = Pubkey::new_unique();
        let no_token_account = SolanaAccount::new(
            Rent::default().minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        program_test.add_account(no_token_pubkey, no_token_account);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let deposit_instruction = deposit_instruction(
            &program_id,
            &result_pubkey,
            &deposit_pubkey,
            &result.yes_mint,
            &yes_token_pubkey,
            &result.no_mint,
            &no_token_pubkey,
            100,
        )
        .unwrap();

        let mut transaction =
            Transaction::new_with_payer(&[deposit_instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let deposit_account = banks_client
            .get_account(deposit_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(0, deposit_account.lamports);
    }
}
