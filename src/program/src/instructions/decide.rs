use super::{ResultAccount, SearchMarketAccount, SearchMarketInstruction};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn decide_instruction(
    program_id: &Pubkey,
    market_pubkey: &Pubkey,
    decision_authority_pubkey: &Pubkey,
    best_result_pubkey: &Pubkey,
) -> Result<Instruction, std::io::Error> {
    let data = SearchMarketInstruction::Decide.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*market_pubkey, false),
        AccountMeta::new_readonly(*decision_authority_pubkey, true),
        AccountMeta::new_readonly(*best_result_pubkey, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn decide(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let market_account_info = next_account_info(account_info_iter)?;
    let decision_authority_info = next_account_info(account_info_iter)?;
    let best_result_info = next_account_info(account_info_iter)?;
    let clock = Clock::get()?;

    if *market_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut market =
        SearchMarketAccount::try_from_slice(&market_account_info.data.borrow()).unwrap();

    if !decision_authority_info.is_signer {
        return Err(ProgramError::InvalidAccountData);
    }

    if *best_result_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let result = ResultAccount::try_from_slice(&best_result_info.data.borrow())?;
    if result.search_market != *market_account_info.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if clock.slot > market.expires_slot {
        return Err(ProgramError::InvalidAccountData);
    }

    if market.decision_authority != *decision_authority_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    market.best_result = *best_result_info.key;

    market
        .serialize(&mut &mut market_account_info.data.borrow_mut()[..])
        .map_err(|e| e.into())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::*;
    use crate::process_instruction;
    use crate::undecided_result;
    use crate::ResultAccount;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_decide() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let decision_authority = Keypair::new();
        let mut market = SearchMarketAccount {
            decision_authority: decision_authority.pubkey(),
            best_result: undecided_result::id(),
            expires_slot: 2,
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

        let some_other_authority = Keypair::new();
        let bad_decide_instruction = decide_instruction(
            &program_id,
            &market_key,
            &some_other_authority.pubkey(),
            &result_key,
        )
        .unwrap();

        let good_decide_instruction = decide_instruction(
            &program_id,
            &market_key,
            &decision_authority.pubkey(),
            &result_key,
        )
        .unwrap();

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        let mut transaction = Transaction::new_with_payer(
            &[
                create_market.clone(),
                create_result.clone(),
                bad_decide_instruction,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(
            &[&payer, &decision_authority, &some_other_authority],
            recent_blockhash,
        );
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();

        let mut transaction = Transaction::new_with_payer(
            &[create_market, create_result, good_decide_instruction],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &decision_authority], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        market.best_result = result_key;
        let market_account = banks_client.get_account(market_key).await.unwrap().unwrap();
        let processed_market =
            SearchMarketAccount::try_from_slice(&market_account.data[..]).unwrap();
        assert_eq!(market, processed_market);
    }

    #[tokio::test]
    async fn test_decide_too_late() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let decision_authority = Keypair::new();
        let market = SearchMarketAccount {
            decision_authority: decision_authority.pubkey(),
            best_result: undecided_result::id(),
            expires_slot: 2,
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

        let decide_instruction = decide_instruction(
            &program_id,
            &market_key,
            &decision_authority.pubkey(),
            &result_key,
        )
        .unwrap();

        let mut context = program_test.start_with_context().await;
        let mut transaction = Transaction::new_with_payer(
            &[create_market.clone(), create_result.clone()],
            Some(&context.payer.pubkey()),
        );
        transaction.sign(
            &[&context.payer, &decision_authority, &decision_authority],
            context.last_blockhash,
        );
        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        context.warp_to_slot(5).unwrap();

        let mut transaction =
            Transaction::new_with_payer(&[decide_instruction], Some(&context.payer.pubkey()));
        transaction.sign(
            &[&context.payer, &decision_authority, &decision_authority],
            context.last_blockhash,
        );
        let error = context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err();
        assert_eq!(error.to_string(), "transport transaction error: Error processing Instruction 0: invalid account data for instruction");
    }
}
