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

pub fn fill_order_instruction(
    program_id: &Pubkey,
    buy_order: &Pubkey,
    buyer_token_account: &Pubkey,
    sell_order: &Pubkey,
    seller_sol_account: &Pubkey,
    execution_authority: &Pubkey,
) -> Result<Instruction, std::io::Error> {
    let (sol_escrow, sol_escrow_bump_seed) =
        Pubkey::find_program_address(&[b"sol_escrow", &buy_order.to_bytes()], program_id);
    let (token_escrow, token_escrow_bump_seed) =
        Pubkey::find_program_address(&[b"token_escrow", &sell_order.to_bytes()], program_id);

    let data = SearchMarketInstruction::FillOrder {
        sol_escrow_bump_seed,
        token_escrow_bump_seed,
    }
    .try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*buy_order, false),
        AccountMeta::new(*buyer_token_account, false),
        AccountMeta::new(*sell_order, false),
        AccountMeta::new(*seller_sol_account, false),
        AccountMeta::new(sol_escrow, false),
        AccountMeta::new(token_escrow, false),
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

pub fn fill_order(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    sol_escrow_bump_seed: u8,
    token_escrow_bump_seed: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let buy_order_account_info = next_account_info(account_info_iter)?;
    let buyer_token_account_info = next_account_info(account_info_iter)?;
    let sell_order_account_info = next_account_info(account_info_iter)?;
    let seller_sol_account_info = next_account_info(account_info_iter)?;
    let sol_escrow_account_info = next_account_info(account_info_iter)?;
    let token_escrow_account_info = next_account_info(account_info_iter)?;
    let execution_authority_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let mut buy_order =
        OrderAccount::try_from_slice(&buy_order_account_info.data.borrow()).unwrap();
    let mut sell_order =
        OrderAccount::try_from_slice(&sell_order_account_info.data.borrow()).unwrap();

    let sol_escrow_seeds: &[&[u8]] = &[
        b"sol_escrow",
        &buy_order_account_info.key.to_bytes(),
        &[sol_escrow_bump_seed],
    ];
    let sol_escrow_key = Pubkey::create_program_address(sol_escrow_seeds, program_id).unwrap();
    if *sol_escrow_account_info.key != sol_escrow_key {
        return Err(ProgramError::InvalidAccountData);
    }

    let token_escrow_seeds: &[&[u8]] = &[
        b"token_escrow",
        &sell_order_account_info.key.to_bytes(),
        &[token_escrow_bump_seed],
    ];
    let token_escrow_key = Pubkey::create_program_address(token_escrow_seeds, program_id).unwrap();
    if *token_escrow_account_info.key != token_escrow_key {
        return Err(ProgramError::InvalidAccountData);
    }

    if buy_order.price < sell_order.price {
        return Err(ProgramError::InvalidAccountData);
    }

    if !(*execution_authority_account_info.key == buy_order.execution_authority
        || *execution_authority_account_info.key == sell_order.execution_authority)
    {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let price = if buy_order.creation_slot <= sell_order.creation_slot {
        buy_order.price
    } else {
        sell_order.price
    };
    let quantity = if buy_order.quantity <= sell_order.quantity {
        buy_order.quantity
    } else {
        sell_order.quantity
    };

    invoke_signed(
        &system_instruction::transfer(
            sol_escrow_account_info.key,
            seller_sol_account_info.key,
            price * quantity,
        ),
        &[
            sol_escrow_account_info.clone(),
            seller_sol_account_info.clone(),
            system_program_info.clone(),
        ],
        &[sol_escrow_seeds],
    )?;

    invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            &token_escrow_account_info.key,
            &buyer_token_account_info.key,
            &token_escrow_account_info.key,
            &[],
            quantity,
        )
        .unwrap(),
        &[
            token_escrow_account_info.clone(),
            buyer_token_account_info.clone(),
            token_escrow_account_info.clone(),
            spl_token_program_info.clone(),
        ],
        &[token_escrow_seeds],
    )?;

    buy_order.quantity -= quantity;
    buy_order
        .serialize(&mut &mut buy_order_account_info.data.borrow_mut()[..])
        .unwrap();

    sell_order.quantity -= quantity;
    sell_order
        .serialize(&mut &mut sell_order_account_info.data.borrow_mut()[..])
        .unwrap();

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::*;
    use crate::process_instruction;
    use crate::{undecided_result, ResultAccount, SearchMarketAccount};
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::Account as SolanaAccount,
        rent::Rent,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    #[tokio::test]
    async fn test_fill_order() {
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

        let seller_sol_pubkey = Pubkey::new_unique();
        let seller_sol_account = SolanaAccount::new(
            Rent::default().minimum_balance(0),
            0,
            &deposit_keypair.pubkey(),
        );
        program_test.add_account(seller_sol_pubkey, seller_sol_account);
        let mut sell_order = OrderAccount {
            search_market: market_key,
            result: result_key,
            sol_account: seller_sol_pubkey,
            token_account: yes_token_pubkey,
            side: OrderSide::Sell,
            price: 500,
            quantity: 100,
            escrow_bump_seed: 0,
            creation_slot: 1,
            execution_authority: deposit_keypair.pubkey(),
        };
        let (sell_order_key, sell_escrow_key, create_sell_order) = setup_order(
            &mut sell_order,
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let (buy_token_pubkey, init_buy_token) = setup_token(
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
        );
        let mut buy_order = OrderAccount {
            search_market: market_key,
            result: result_key,
            sol_account: deposit_keypair.pubkey(),
            token_account: buy_token_pubkey,
            side: OrderSide::Buy,
            price: 501,
            quantity: 100,
            escrow_bump_seed: 0,
            creation_slot: 1,
            execution_authority: deposit_keypair.pubkey(),
        };
        let (buy_order_key, buy_escrow_key, create_buy_order) = setup_order(
            &mut buy_order,
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            &mut program_test,
            &program_id,
        );

        let fill_order = fill_order_instruction(
            &program_id,
            &buy_order_key,
            &buy_order.token_account,
            &sell_order_key,
            &sell_order.sol_account,
            &buy_order.execution_authority,
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
                create_sell_order,
                init_buy_token,
                create_buy_order,
                fill_order,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(
            &[&payer, &decision_authority, &deposit_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let seller_sol_account = banks_client
            .get_account(seller_sol_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            seller_sol_account.lamports,
            (501 * 100) + Rent::default().minimum_balance(0)
        );

        let buy_token_account = banks_client
            .get_account(buy_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let buy_token = spl_token::state::Account::unpack(&buy_token_account.data).unwrap();
        assert_eq!(buy_token.amount, 100);

        let buy_order_account = banks_client
            .get_account(buy_order_key)
            .await
            .unwrap()
            .unwrap();
        let buy_order = OrderAccount::try_from_slice(&buy_order_account.data).unwrap();
        assert_eq!(buy_order.quantity, 0);

        let sell_order_account = banks_client
            .get_account(sell_order_key)
            .await
            .unwrap()
            .unwrap();
        let sell_order = OrderAccount::try_from_slice(&sell_order_account.data).unwrap();
        assert_eq!(sell_order.quantity, 0);
    }
}
