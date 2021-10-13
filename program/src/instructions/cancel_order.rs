use super::{OrderAccount, OrderSide, SearchMarketInstruction};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, system_program,
};

pub fn cancel_order_instruction(
    program_id: &Pubkey,
    order: &Pubkey,
    side: OrderSide,
    sol_account: &Pubkey,
    token_account: &Pubkey,
    execution_authority: &Pubkey,
) -> Result<Instruction, std::io::Error> {
    let escrow_name: &[u8] = match side {
        OrderSide::Buy => b"sol_escrow",
        OrderSide::Sell => b"token_escrow",
    };
    let (escrow_key, bump_seed) =
        Pubkey::find_program_address(&[escrow_name, &order.to_bytes()], program_id);

    let data = SearchMarketInstruction::CancelOrder.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*order, false),
        AccountMeta::new(*sol_account, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(escrow_key, false),
        AccountMeta::new_readonly(*execution_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn cancel_order(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let order_account_info = next_account_info(account_info_iter)?;
    let sol_account_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let execution_authority_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let order = OrderAccount::try_from_slice(*order_account_info.data.borrow()).unwrap();
    solana_program::msg!("{:?}", sol_account_info);
    match order.side {
        OrderSide::Buy => invoke_signed(
            &system_instruction::transfer(
                escrow_account_info.key,
                sol_account_info.key,
                order.quantity * order.price,
            ),
            &[
                escrow_account_info.clone(),
                sol_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&[
                b"sol_escrow",
                &order_account_info.key.to_bytes(),
                &[order.escrow_bump_seed],
            ]],
        )?,
        OrderSide::Sell => invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                &escrow_account_info.key,
                &token_account_info.key,
                &escrow_account_info.key,
                &[],
                order.quantity,
            )
            .unwrap(),
            &[
                escrow_account_info.clone(),
                token_account_info.clone(),
                escrow_account_info.clone(),
                spl_token_program_info.clone(),
            ],
            &[&[
                b"token_escrow",
                &order_account_info.key.to_bytes(),
                &[order.escrow_bump_seed],
            ]],
        )?,
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::*;
    use crate::process_instruction;
    use crate::test_utils::*;
    use crate::{ResultAccount, SearchMarketAccount};
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::account::ReadableAccount;
    use solana_sdk::{
        account::Account as SolanaAccount,
        rent::Rent,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_cancel_order_sell() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let decision_authority = Keypair::new();
        let market =
            SearchMarketAccount::new(decision_authority.pubkey(), "cyberpunk".to_string(), 2);
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

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

        let mut order = OrderAccount::new(
            market_key,
            result_key,
            deposit_keypair.pubkey(),
            yes_token_pubkey,
            OrderSide::Sell,
            500,
            100,
            0,
            1,
            deposit_keypair.pubkey(),
        );
        let (order_key, escrow_key, create_order) = setup_order(
            &mut order,
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let cancel_order = cancel_order_instruction(
            &program_id,
            &order_key,
            order.side,
            &order.sol_account,
            &order.token_account,
            &order.execution_authority,
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
                create_order,
                cancel_order,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(
            &[&payer, &decision_authority, &deposit_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();
        let yes_token_account = banks_client
            .get_account(yes_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let yes_token = spl_token::state::Account::unpack(&yes_token_account.data()).unwrap();
        assert_eq!(yes_token.amount, 100);
    }

    #[tokio::test]
    async fn test_cancel_order_buy() {
        let program_id = crate::id();
        let mut program_test =
            ProgramTest::new("askbid", program_id, processor!(process_instruction));

        let decision_authority = Keypair::new();
        let market =
            SearchMarketAccount::new(decision_authority.pubkey(), "cyberpunk".to_string(), 2);
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

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

        let sol_account_keypair = Keypair::new();
        let sol_account = SolanaAccount::new(
            Rent::default().minimum_balance(0) + 500 * 100,
            0,
            &system_program::id(),
        );
        program_test.add_account(sol_account_keypair.pubkey(), sol_account);
        let mut order = OrderAccount::new(
            market_key,
            result_key,
            sol_account_keypair.pubkey(),
            yes_token_pubkey,
            OrderSide::Buy,
            500,
            100,
            0,
            1,
            deposit_keypair.pubkey(),
        );
        let (order_key, escrow_key, create_order) = setup_order(
            &mut order,
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let cancel_order = cancel_order_instruction(
            &program_id,
            &order_key,
            order.side,
            &order.sol_account,
            &order.token_account,
            &order.execution_authority,
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
                create_order,
                cancel_order,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(
            &[
                &payer,
                &decision_authority,
                &deposit_keypair,
                &sol_account_keypair,
            ],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let escrow_account = banks_client.get_account(escrow_key).await.unwrap().unwrap();
        assert_eq!(escrow_account.lamports, Rent::default().minimum_balance(0));

        let sol_account = banks_client
            .get_account(order.sol_account)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            sol_account.lamports,
            Rent::default().minimum_balance(0) + order.quantity * order.price
        );
    }
}
