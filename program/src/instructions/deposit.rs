use super::{ResultAccount, SearchMarketAccount, SearchMarketInstruction};
use crate::LAMPORTS_PER_TOKEN;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer,
    system_program,
    sysvar::Sysvar,
};

pub fn deposit_instruction(
    program_id: &Pubkey,
    market_pubkey: &Pubkey,
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
        AccountMeta::new_readonly(*market_pubkey, false),
        AccountMeta::new_readonly(*result_pubkey, false),
        AccountMeta::new(*deposit_pubkey, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(mint_authority_key, false),
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

pub fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let market_account_info = next_account_info(account_info_iter)?;
    msg!("Market {}", market_account_info.key.to_string());
    let result_account_info = next_account_info(account_info_iter)?;
    msg!("result {}", result_account_info.key.to_string());
    let deposit_account_info = next_account_info(account_info_iter)?;
    msg!("deposit {}", deposit_account_info.key.to_string());
    let system_program_info = next_account_info(account_info_iter)?;
    msg!("system {}", system_program_info.key.to_string());
    let spl_token_program_info = next_account_info(account_info_iter)?;
    msg!("spl {}", spl_token_program_info.key.to_string());
    let mint_authority_info = next_account_info(account_info_iter)?;
    msg!("mint auth {}", mint_authority_info.key.to_string());
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    msg!("yes mint {}", yes_mint_account_info.key.to_string());
    let yes_token_account_info = next_account_info(account_info_iter)?;
    msg!("yes token {}", yes_token_account_info.key.to_string());
    let no_mint_account_info = next_account_info(account_info_iter)?;
    msg!("no mint {}", no_mint_account_info.key.to_string());
    let no_token_account_info = next_account_info(account_info_iter)?;
    msg!("no token {}", no_token_account_info.key.to_string());
    let clock = Clock::get()?;

    if *market_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let market = SearchMarketAccount::try_from_slice(*market_account_info.data.borrow())?;
    if clock.slot > market.expires_slot {
        return Err(ProgramError::InvalidAccountData);
    }

    if *result_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let result = ResultAccount::try_from_slice(&result_account_info.data.borrow())?;
    if result.search_market != *market_account_info.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if *system_program_info.key != system_program::id() {
        return Err(ProgramError::InvalidAccountData);
    }

    if *spl_token_program_info.key != spl_token::id() {
        return Err(ProgramError::InvalidAccountData);
    }

    if Pubkey::create_program_address(&[b"mint_authority", &[result.bump_seed]], program_id)?
        != *mint_authority_info.key
    {
        return Err(ProgramError::InvalidArgument);
    }

    if *yes_mint_account_info.key != result.yes_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    if *no_mint_account_info.key != result.no_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    msg!("transfer sol");
    invoke_signed(
        &transfer(
            deposit_account_info.key,
            mint_authority_info.key,
            amount * LAMPORTS_PER_TOKEN,
        ),
        &[
            deposit_account_info.clone(),
            mint_authority_info.clone(),
            system_program_info.clone(),
        ],
        &[&[b"mint_authority", &[result.bump_seed]]],
    )?;

    msg!("mint yes");
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

    msg!("mint no");
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

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::*;
    use crate::process_instruction;
    use crate::test_utils::*;
    use crate::ResultAccount;
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::Account as SolanaAccount,
        rent::Rent,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use spl_token::state::Account;

    pub fn setup_token(
        mint: &Pubkey,
        owner: &Pubkey,
        program_test: &mut ProgramTest,
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

    pub fn setup_deposit(
        deposit_key: &Pubkey,
        amount: u64,
        market_key: &Pubkey,
        result_key: &Pubkey,
        result: &ResultAccount,
        yes_token_pubkey: &Pubkey,
        no_token_pubkey: &Pubkey,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> Instruction {
        let deposit_min_balance = Rent::default().minimum_balance(0);
        let deposit_account = SolanaAccount::new(
            deposit_min_balance + amount * LAMPORTS_PER_TOKEN,
            0,
            &system_program::id(),
        );
        program_test.add_account(*deposit_key, deposit_account);

        let deposit_instruction = deposit_instruction(
            &program_id,
            &market_key,
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

        let decision_authority = Keypair::new();
        let market =
            SearchMarketAccount::new(decision_authority.pubkey(), "cyberpunk".to_string(), 1);
        let (market_key, create_market) = setup_market(&market, 1, &mut program_test, &program_id);

        let mut result = ResultAccount::new(
            market_key,
            String::from("http://cyberpunk.net"),
            String::from("Cyberpunk website"),
            String::from("A game fated to be legend"),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
        );
        let (result_key, create_result) = setup_result(&mut result, &mut program_test, &program_id);
        let result_min_balance = minimum_balance(&result).unwrap();

        let deposit_keypair = Keypair::new();
        let (yes_token_pubkey, init_yes_token) = setup_token(
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
        );

        let (no_token_pubkey, init_no_token) = setup_token(
            &result.no_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
        );

        let deposit_min_balance = Rent::default().minimum_balance(0);
        let deposit_instruction = setup_deposit(
            &deposit_keypair.pubkey(),
            100,
            &market_key,
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
        transaction.sign(
            &[&payer, &decision_authority, &deposit_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let result_account = banks_client.get_account(result_key).await.unwrap().unwrap();
        assert_eq!(result_min_balance, result_account.lamports);

        let mint_authority_key =
            Pubkey::create_program_address(&[b"mint_authority", &[result.bump_seed]], &program_id)
                .unwrap();
        let mint_authority_account = banks_client
            .get_account(mint_authority_key)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(mint_authority_account.lamports, 100 * LAMPORTS_PER_TOKEN);

        let deposit_account = banks_client
            .get_account(deposit_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();
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
}
