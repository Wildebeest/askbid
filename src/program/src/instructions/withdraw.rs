use super::{ResultAccount, SearchMarketAccount, SearchMarketInstruction};
use crate::undecided_result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer,
    system_program,
    sysvar::Sysvar,
};

pub fn withdraw_instruction(
    program_id: &Pubkey,
    market_pubkey: &Pubkey,
    result_pubkey: &Pubkey,
    withdraw_pubkey: &Pubkey,
    token_owner_pubkey: &Pubkey,
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
        AccountMeta::new_readonly(*market_pubkey, false),
        AccountMeta::new_readonly(*result_pubkey, false),
        AccountMeta::new(*withdraw_pubkey, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(mint_authority_key, false),
        AccountMeta::new_readonly(*token_owner_pubkey, true),
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

pub fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let market_account_info = next_account_info(account_info_iter)?;
    let result_account_info = next_account_info(account_info_iter)?;
    let withdraw_account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let token_owner_info = next_account_info(account_info_iter)?;
    let yes_mint_account_info = next_account_info(account_info_iter)?;
    let yes_token_account_info = next_account_info(account_info_iter)?;
    let no_mint_account_info = next_account_info(account_info_iter)?;
    let no_token_account_info = next_account_info(account_info_iter)?;
    let clock = Clock::get()?;

    if *market_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let market = SearchMarketAccount::try_from_slice(*market_account_info.data.borrow())?;

    if *result_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let result = ResultAccount::try_from_slice(&result_account_info.data.borrow())?;

    if !withdraw_account_info.is_signer {
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
        return Err(ProgramError::InvalidAccountData);
    }

    if *yes_mint_account_info.key != result.yes_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    if *no_mint_account_info.key != result.no_mint {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut withdraw_amount = amount;
    let mut yes_amount = 0;
    let mut no_amount = 0;
    if market.best_result == undecided_result::id() {
        yes_amount = amount;
        no_amount = amount;
    } else if market.best_result == *result_account_info.key {
        yes_amount = amount;
    } else {
        no_amount = amount;
    }

    invoke_signed(
        &transfer(
            mint_authority_info.key,
            withdraw_account_info.key,
            withdraw_amount,
        ),
        &[
            mint_authority_info.clone(),
            withdraw_account_info.clone(),
            system_program_info.clone(),
        ],
        &[&[b"mint_authority", &[result.bump_seed]]],
    )?;

    if yes_amount > 0 {
        invoke(
            &spl_token::instruction::burn(
                spl_token_program_info.key,
                yes_token_account_info.key,
                yes_mint_account_info.key,
                token_owner_info.key,
                &[],
                yes_amount,
            )?,
            &[
                yes_token_account_info.clone(),
                yes_mint_account_info.clone(),
                token_owner_info.clone(),
                spl_token_program_info.clone(),
            ],
        )?;
    }

    if no_amount > 0 {
        invoke(
            &spl_token::instruction::burn(
                spl_token_program_info.key,
                no_token_account_info.key,
                no_mint_account_info.key,
                token_owner_info.key,
                &[],
                amount,
            )?,
            &[
                no_token_account_info.clone(),
                no_mint_account_info.clone(),
                token_owner_info.clone(),
                spl_token_program_info.clone(),
            ],
        )?;
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::*;
    use crate::process_instruction;
    use crate::undecided_result;
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

    #[tokio::test]
    async fn test_withdraw() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let decision_authority = Keypair::new();
        let market = SearchMarketAccount {
            decision_authority: decision_authority.pubkey(),
            best_result: undecided_result::id(),
            expires_slot: 1,
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

        let withdraw_keypair = Keypair::new();
        let withdraw_account =
            SolanaAccount::new(Rent::default().minimum_balance(0), 0, &system_program::id());
        program_test.add_account(withdraw_keypair.pubkey(), withdraw_account);
        let withdraw_instruction = withdraw_instruction(
            &program_id,
            &market_key,
            &result_key,
            &withdraw_keypair.pubkey(),
            &deposit_keypair.pubkey(),
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
        transaction.sign(
            &[
                &payer,
                &decision_authority,
                &deposit_keypair,
                &withdraw_keypair,
            ],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

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
