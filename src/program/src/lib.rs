use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    clock::Slot,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::transfer,
    system_program, sysvar,
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
    let (mint_authority_key, bump_seed) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);
    let data = SearchMarketInstruction::CreateResult {
        url,
        name,
        snippet,
        bump_seed,
    }
    .try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*result_pubkey, false),
        AccountMeta::new_readonly(*market_pubkey, false),
        AccountMeta::new(*yes_mint_pubkey, false),
        AccountMeta::new(*no_mint_pubkey, false),
        AccountMeta::new_readonly(mint_authority_key, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
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
    let (mint_authority_key, _bump_seed) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);
    let data = SearchMarketInstruction::Deposit { amount }.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*result_pubkey, false),
        AccountMeta::new(*deposit_pubkey, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(mint_authority_key, false),
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

pub fn withdraw_instruction(
    program_id: &Pubkey,
    result_pubkey: &Pubkey,
    withdraw_pubkey: &Pubkey,
    yes_mint_pubkey: &Pubkey,
    yes_token_pubkey: &Pubkey,
    no_mint_pubkey: &Pubkey,
    no_token_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction, std::io::Error> {
    let (mint_authority_key, _bump_seed) =
        Pubkey::find_program_address(&[b"mint_authority"], program_id);
    let data = SearchMarketInstruction::Withdraw { amount }.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*result_pubkey, false),
        AccountMeta::new(*withdraw_pubkey, false),
        AccountMeta::new_readonly(system_program::id(), false),
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
impl std::fmt::Display for SearchMarketAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.search_string,
            self.expires_slot,
            self.best_result.is_some()
        )
    }
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
    pub bump_seed: u8,
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

    let result = search_market
        .serialize(&mut &mut market_account_info.data.borrow_mut()[..])
        .map_err(|e| e.into());

    return result;
}

pub fn create_result(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    url: String,
    name: String,
    snippet: String,
    bump_seed: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let market_account_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    let spl_token_account_info = next_account_info(account_info_iter)?;

    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            yes_mint_account_info.key,
            mint_authority_info.key,
            None,
            spl_token::native_mint::DECIMALS,
        )?,
        &[
            yes_mint_account_info.clone(),
            rent_account_info.clone(),
            spl_token_account_info.clone(),
        ],
    )?;

    invoke(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            no_mint_account_info.key,
            mint_authority_info.key,
            None,
            spl_token::native_mint::DECIMALS,
        )?,
        &[
            no_mint_account_info.clone(),
            rent_account_info.clone(),
            spl_token_account_info.clone(),
        ],
    )?;

    let result = ResultAccount {
        search_market: *market_account_info.key,
        url,
        name,
        snippet,
        yes_mint: *yes_mint_account_info.key,
        no_mint: *no_mint_account_info.key,
        bump_seed,
    };

    result
        .serialize(&mut &mut result_account_info.data.borrow_mut()[..])
        .map_err(|e| e.into())
}

pub fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let deposit_account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let yes_token_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let no_token_account_info = next_account_info(account_info_iter)?;

    let result = ResultAccount::try_from_slice(&result_account_info.data.borrow())?;

    invoke(
        &transfer(deposit_account_info.key, result_account_info.key, amount),
        &[
            deposit_account_info.clone(),
            result_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            spl_token_program_info.key,
            yes_mint_account_info.key,
            yes_token_account_info.key,
            mint_authority_info.key,
            &[],
            amount,
        )?,
        &[
            yes_mint_account_info.clone(),
            yes_token_account_info.clone(),
            mint_authority_info.clone(),
            spl_token_program_info.clone(),
        ],
        &[&[b"mint_authority", &[result.bump_seed]]],
    )?;

    invoke_signed(
        &spl_token::instruction::mint_to(
            spl_token_program_info.key,
            no_mint_account_info.key,
            no_token_account_info.key,
            mint_authority_info.key,
            &[],
            amount,
        )?,
        &[
            no_mint_account_info.clone(),
            no_token_account_info.clone(),
            mint_authority_info.clone(),
            spl_token_program_info.clone(),
        ],
        &[&[b"mint_authority", &[result.bump_seed]]],
    )?;

    Ok(())
}

pub fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let result_account_info = next_account_info(account_info_iter)?;
    let withdraw_account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let yes_token_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let no_token_account_info = next_account_info(account_info_iter)?;

    let result = ResultAccount::try_from_slice(&result_account_info.data.borrow())?;
    
    invoke(
        &transfer(result_account_info.key, withdraw_account_info.key, amount),
        &[
            result_account_info.clone(),
            withdraw_account_info.clone(),
            system_program_info.clone(),
        ],
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
        SearchMarketInstruction::CreateResult {
            url,
            name,
            snippet,
            bump_seed,
        } => create_result(program_id, accounts, url, name, snippet, bump_seed),
        SearchMarketInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        SearchMarketInstruction::Withdraw { amount } => withdraw(program_id, accounts, amount),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::Account as SolanaAccount,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use spl_token::state::{Account, Mint};

    fn setup_market(
        market: &SearchMarketAccount,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> (Pubkey, Instruction) {
        let market_key = Pubkey::new_unique();
        let market_account = SolanaAccount::new(
            minimum_balance(&market).unwrap(),
            space(&market).unwrap(),
            program_id,
        );
        program_test.add_account(market_key, market_account);

        let instruction = create_market_instruction(
            program_id,
            &market_key,
            market.expires_slot,
            market.search_string.clone(),
        )
        .unwrap();
        return (market_key, instruction);
    }

    #[tokio::test]
    async fn test_create_market() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction = Transaction::new_with_payer(&[create_market], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let market_account = banks_client.get_account(market_key).await.unwrap().unwrap();
        let processed_market =
            SearchMarketAccount::try_from_slice(&market_account.data[..]).unwrap();
        assert_eq!(market, processed_market);
    }

    fn setup_result(
        result: &mut ResultAccount,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> (Pubkey, Instruction) {
        let result_key = Pubkey::new_unique();
        let (_mint_authority_key, bump_seed) =
            Pubkey::find_program_address(&[b"mint_authority"], &program_id);
        result.bump_seed = bump_seed;

        let result_account = SolanaAccount::new(
            minimum_balance(result).unwrap(),
            space(result).unwrap(),
            program_id,
        );
        program_test.add_account(result_key, result_account);

        let yes_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &spl_token::id(),
        );
        program_test.add_account(result.yes_mint, yes_mint_account);

        let no_mint_account = SolanaAccount::new(
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN,
            &spl_token::id(),
        );
        program_test.add_account(result.no_mint, no_mint_account);

        let create_result_instruction = create_result_instruction(
            &program_id,
            &result_key,
            &result.search_market,
            &result.yes_mint,
            &result.no_mint,
            result.url.clone(),
            result.name.clone(),
            result.snippet.clone(),
        )
        .unwrap();

        return (result_key, create_result_instruction);
    }

    #[tokio::test]
    async fn test_create_result() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

        let mut result = ResultAccount {
            search_market: market_key,
            url: String::from("http://cyberpunk.net"),
            name: String::from("Cyberpunk website"),
            snippet: String::from("A game fated to be legend"),
            yes_mint: Pubkey::new_unique(),
            no_mint: Pubkey::new_unique(),
            bump_seed: 0,
        };
        let (result_key, create_result) = setup_result(&mut result, &mut program_test, &program_id);
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction =
            Transaction::new_with_payer(&[create_market, create_result], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let result_account = banks_client.get_account(result_key).await.unwrap().unwrap();
        let processed_result = ResultAccount::try_from_slice(&result_account.data[..]).unwrap();
        assert_eq!(result, processed_result);

        let yes_mint_account = banks_client
            .get_account(result.yes_mint)
            .await
            .unwrap()
            .unwrap();
        let processed_mint = Mint::unpack_from_slice(&yes_mint_account.data[..]).unwrap();
        assert_eq!(true, processed_mint.is_initialized);

        let no_mint_account = banks_client
            .get_account(result.no_mint)
            .await
            .unwrap()
            .unwrap();
        let processed_mint = Mint::unpack_from_slice(&no_mint_account.data[..]).unwrap();
        assert_eq!(true, processed_mint.is_initialized);
    }

    fn setup_token(
        mint: &Pubkey,
        owner: &Pubkey,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> (Pubkey, Instruction) {
        let token_pubkey = Pubkey::new_unique();
        let token_account = SolanaAccount::new(
            Rent::default().minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        program_test.add_account(token_pubkey, token_account);
        let init_token = spl_token::instruction::initialize_account(
            &spl_token::id(),
            &token_pubkey,
            mint,
            owner,
        )
        .unwrap();

        return (token_pubkey, init_token);
    }

    fn setup_deposit(
        deposit_key: &Pubkey,
        amount: u64,
        result_key: &Pubkey,
        result: &ResultAccount,
        yes_token_pubkey: &Pubkey,
        no_token_pubkey: &Pubkey,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> Instruction {
        let deposit_min_balance = Rent::default().minimum_balance(0);
        let deposit_account =
            SolanaAccount::new(deposit_min_balance + amount, 0, &system_program::id());
        program_test.add_account(*deposit_key, deposit_account);

        let deposit_instruction = deposit_instruction(
            &program_id,
            &result_key,
            &deposit_key,
            &result.yes_mint,
            &yes_token_pubkey,
            &result.no_mint,
            &no_token_pubkey,
            amount,
        )
        .unwrap();

        return deposit_instruction;
    }

    #[tokio::test]
    async fn test_deposit() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

        let mut result = ResultAccount {
            search_market: market_key,
            url: String::from("http://cyberpunk.net"),
            name: String::from("Cyberpunk website"),
            snippet: String::from("A game fated to be legend"),
            yes_mint: Pubkey::new_unique(),
            no_mint: Pubkey::new_unique(),
            bump_seed: 0,
        };
        let (result_key, create_result) = setup_result(&mut result, &mut program_test, &program_id);
        let result_min_balance = minimum_balance(&result).unwrap();

        let deposit_keypair = Keypair::new();
        let (yes_token_pubkey, init_yes_token) = setup_token(
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let (no_token_pubkey, init_no_token) = setup_token(
            &result.no_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let deposit_min_balance = Rent::default().minimum_balance(0);
        let deposit_instruction = setup_deposit(
            &deposit_keypair.pubkey(),
            100,
            &result_key,
            &result,
            &yes_token_pubkey,
            &no_token_pubkey,
            &mut program_test,
            &program_id,
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let mut transaction = Transaction::new_with_payer(
            &[
                create_market,
                create_result,
                init_yes_token,
                init_no_token,
                deposit_instruction,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &deposit_keypair], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let result_account = banks_client.get_account(result_key).await.unwrap().unwrap();

        let deposit_account = banks_client
            .get_account(deposit_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result_min_balance + 100, result_account.lamports);
        assert_eq!(deposit_min_balance, deposit_account.lamports);

        let yes_token_account = banks_client
            .get_account(yes_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let yes_token_data = Account::unpack_from_slice(&yes_token_account.data).unwrap();
        assert_eq!(yes_token_data.amount, 100);

        let no_token_account = banks_client
            .get_account(no_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let no_token_data = Account::unpack_from_slice(&no_token_account.data).unwrap();
        assert_eq!(no_token_data.amount, 100);
    }

    #[tokio::test]
    async fn test_withdraw() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let market = SearchMarketAccount {
            best_result: None,
            expires_slot: 0,
            search_string: "cyberpunk".to_string(),
        };
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

        let mut result = ResultAccount {
            search_market: market_key,
            url: String::from("http://cyberpunk.net"),
            name: String::from("Cyberpunk website"),
            snippet: String::from("A game fated to be legend"),
            yes_mint: Pubkey::new_unique(),
            no_mint: Pubkey::new_unique(),
            bump_seed: 0,
        };
        let (result_key, create_result) = setup_result(&mut result, &mut program_test, &program_id);
        let result_min_balance = minimum_balance(&result).unwrap();

        let deposit_keypair = Keypair::new();
        let (yes_token_pubkey, init_yes_token) = setup_token(
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let (no_token_pubkey, init_no_token) = setup_token(
            &result.no_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let deposit_instruction = setup_deposit(
            &deposit_keypair.pubkey(),
            100,
            &result_key,
            &result,
            &yes_token_pubkey,
            &no_token_pubkey,
            &mut program_test,
            &program_id,
        );

        let withdraw_keypair = Keypair::new();
        let withdraw_instruction = withdraw_instruction(
            &program_id,
            &result_key,
            &withdraw_keypair.pubkey(),
            &result.yes_mint,
            &yes_token_pubkey,
            &result.no_mint,
            &no_token_pubkey,
            99,
        )
        .unwrap();

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let mut transaction = Transaction::new_with_payer(
            &[
                create_market,
                create_result,
                init_yes_token,
                init_no_token,
                deposit_instruction,
                withdraw_instruction,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &deposit_keypair], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let result_account = banks_client.get_account(result_key).await.unwrap().unwrap();
        assert_eq!(result_min_balance + 1, result_account.lamports);

        let withdraw_min_balance = Rent::default().minimum_balance(0);
        let withdraw_account = banks_client
            .get_account(withdraw_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(withdraw_min_balance + 99, withdraw_account.lamports);

        let yes_token_account = banks_client
            .get_account(yes_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let yes_token_data = Account::unpack_from_slice(&yes_token_account.data).unwrap();
        assert_eq!(yes_token_data.amount, 1);

        let no_token_account = banks_client
            .get_account(no_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let no_token_data = Account::unpack_from_slice(&no_token_account.data).unwrap();
        assert_eq!(no_token_data.amount, 1);
    }
}
