use super::{SearchMarketAccount, SearchMarketInstruction};
use crate::instructions::AccountType;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{Clock, Slot},
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction, system_program,
    sysvar::{rent, Sysvar},
};

pub fn create_order_instruction(
    program_id: &Pubkey,
    order: &Pubkey,
    search_market: &Pubkey,
    result: &Pubkey,
    sol_account: &Pubkey,
    token_account: &Pubkey,
    token_mint_account: &Pubkey,
    token_authority_account: &Pubkey,
    execution_authority: &Pubkey,
    side: OrderSide,
    price: u64,
    quantity: u64,
) -> Result<Instruction, std::io::Error> {
    let escrow_name: &[u8] = match side {
        OrderSide::Buy => b"sol_escrow",
        OrderSide::Sell => b"token_escrow",
    };
    let (escrow_key, bump_seed) =
        Pubkey::find_program_address(&[escrow_name, &order.to_bytes()], program_id);
    let is_buy_side = side == OrderSide::Buy;
    let data = SearchMarketInstruction::CreateOrder {
        side,
        price,
        quantity,
        escrow_bump_seed: bump_seed,
    }
    .try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*order, false),
        AccountMeta::new_readonly(*search_market, false),
        AccountMeta::new_readonly(*result, false),
        AccountMeta::new(*sol_account, is_buy_side),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*token_mint_account, false),
        AccountMeta::new_readonly(*token_authority_account, !is_buy_side),
        AccountMeta::new(escrow_key, false),
        AccountMeta::new_readonly(*execution_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct OrderAccount {
    pub account_type: AccountType,
    pub account_version: u8,
    pub search_market: Pubkey,
    pub result: Pubkey,
    pub sol_account: Pubkey,
    pub token_account: Pubkey,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u64,
    pub escrow_bump_seed: u8,
    pub creation_slot: Slot,
    pub execution_authority: Pubkey,
}

impl OrderAccount {
    pub fn new(
        search_market: Pubkey,
        result: Pubkey,
        sol_account: Pubkey,
        token_account: Pubkey,
        side: OrderSide,
        price: u64,
        quantity: u64,
        escrow_bump_seed: u8,
        creation_slot: Slot,
        execution_authority: Pubkey,
    ) -> OrderAccount {
        OrderAccount {
            account_type: AccountType::Order,
            account_version: 0,
            search_market,
            result,
            sol_account,
            token_account,
            side,
            price,
            quantity,
            escrow_bump_seed,
            creation_slot,
            execution_authority,
        }
    }
}

pub fn create_order(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    side: OrderSide,
    price: u64,
    quantity: u64,
    escrow_bump_seed: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let order_account_info = next_account_info(account_info_iter)?;
    let market_account_info = next_account_info(account_info_iter)?;
    let result_account_info = next_account_info(account_info_iter)?;
    let sol_account_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let token_mint_account_info = next_account_info(account_info_iter)?;
    let token_authority_account_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let execution_authority_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let clock = Clock::get()?;

    if *spl_token_program_info.key != spl_token::id() {
        return Err(ProgramError::InvalidArgument);
    }

    if *rent_account_info.key != rent::id() {
        return Err(ProgramError::InvalidArgument);
    }

    let escrow_owner = if side == OrderSide::Buy {
        system_program::id()
    } else {
        spl_token::id()
    };
    if *escrow_account_info.owner != escrow_owner {
        return Err(ProgramError::IllegalOwner);
    }

    match side {
        OrderSide::Buy => {
            let escrow_pubkey = Pubkey::create_program_address(
                &[
                    b"sol_escrow",
                    &order_account_info.key.to_bytes(),
                    &[escrow_bump_seed],
                ],
                program_id,
            )
            .unwrap();
            if *escrow_account_info.key != escrow_pubkey {
                return Err(ProgramError::InvalidAccountData);
            }

            invoke(
                &system_instruction::transfer(
                    sol_account_info.key,
                    &escrow_account_info.key,
                    price * quantity,
                ),
                &[
                    sol_account_info.clone(),
                    escrow_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }
        OrderSide::Sell => {
            let escrow_pubkey = Pubkey::create_program_address(
                &[
                    b"token_escrow",
                    &order_account_info.key.to_bytes(),
                    &[escrow_bump_seed],
                ],
                program_id,
            )
            .unwrap();
            if *escrow_account_info.key != escrow_pubkey {
                return Err(ProgramError::InvalidAccountData);
            }

            let initialize_account = &spl_token::instruction::initialize_account2(
                &spl_token::id(),
                &escrow_account_info.key,
                &token_mint_account_info.key,
                &escrow_account_info.key,
            )
            .unwrap();

            invoke(
                initialize_account,
                &[
                    escrow_account_info.clone(),
                    token_mint_account_info.clone(),
                    rent_account_info.clone(),
                    spl_token_program_info.clone(),
                ],
            )?;

            invoke(
                &spl_token::instruction::transfer(
                    &spl_token::id(),
                    &token_account_info.key,
                    &escrow_pubkey,
                    &token_authority_account_info.key,
                    &[],
                    quantity,
                )
                .unwrap(),
                &[
                    token_account_info.clone(),
                    escrow_account_info.clone(),
                    token_authority_account_info.clone(),
                    spl_token_program_info.clone(),
                ],
            )?;
        }
    }

    let order = OrderAccount::new(
        *market_account_info.key,
        *result_account_info.key,
        *sol_account_info.key,
        *token_account_info.key,
        side,
        price,
        quantity,
        escrow_bump_seed,
        clock.slot,
        *execution_authority_account_info.key,
    );

    order
        .serialize(&mut &mut order_account_info.data.borrow_mut()[..])
        .map(|_| ())
        .map_err(|e| e.into())
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

    pub fn setup_order(
        order: &mut OrderAccount,
        token_mint_account: &Pubkey,
        token_authority_account: &Pubkey,
        program_test: &mut ProgramTest,
        program_id: &Pubkey,
    ) -> (Pubkey, Pubkey, Instruction) {
        let order_key = Pubkey::new_unique();
        let (escrow_key, bump_seed, escrow_account) = match order.side {
            OrderSide::Buy => {
                let (escrow_key, bump_seed) = Pubkey::find_program_address(
                    &[b"sol_escrow", &order_key.to_bytes()],
                    &program_id,
                );
                let escrow_account = SolanaAccount::new(
                    Rent::default().minimum_balance(0),
                    0,
                    &system_program::id(),
                );
                (escrow_key, bump_seed, escrow_account)
            }
            OrderSide::Sell => {
                let (escrow_key, bump_seed) = Pubkey::find_program_address(
                    &[b"token_escrow", &order_key.to_bytes()],
                    &program_id,
                );
                let escrow_account = SolanaAccount::new(
                    Rent::default().minimum_balance(spl_token::state::Account::LEN),
                    spl_token::state::Account::LEN,
                    &spl_token::id(),
                );
                (escrow_key, bump_seed, escrow_account)
            }
        };
        program_test.add_account(escrow_key, escrow_account);
        order.escrow_bump_seed = bump_seed;

        let order_space = space(order).unwrap();
        let order_min_balance = minimum_balance(order).unwrap();
        let order_account = SolanaAccount::new(order_min_balance, order_space, &program_id);
        program_test.add_account(order_key, order_account);
        let create_order = create_order_instruction(
            &program_id,
            &order_key,
            &order.search_market,
            &order.result,
            &order.sol_account,
            &order.token_account,
            token_mint_account,
            token_authority_account,
            &order.execution_authority,
            order.side,
            order.price,
            order.quantity,
        )
        .unwrap();

        return (order_key, escrow_key, create_order);
    }

    #[tokio::test]
    async fn test_create_order_sell() {
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

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction = Transaction::new_with_payer(
            &[
                create_market,
                create_result,
                init_yes_token,
                init_no_token,
                deposit_instruction,
                create_order,
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(
            &[&payer, &decision_authority, &deposit_keypair],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        let order_account = banks_client.get_account(order_key).await.unwrap().unwrap();
        let processed_order = OrderAccount::try_from_slice(&order_account.data[..]).unwrap();
        assert_eq!(order, processed_order);

        let yes_token_account = banks_client
            .get_account(yes_token_pubkey)
            .await
            .unwrap()
            .unwrap();
        let yes_token_data = Account::unpack_from_slice(&yes_token_account.data).unwrap();
        assert_eq!(yes_token_data.amount, 0);
    }

    #[tokio::test]
    async fn test_create_order_buy() {
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

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction = Transaction::new_with_payer(
            &[
                create_market,
                create_result,
                init_yes_token,
                init_no_token,
                deposit_instruction,
                create_order,
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

        let order_account = banks_client.get_account(order_key).await.unwrap().unwrap();
        let processed_order = OrderAccount::try_from_slice(&order_account.data[..]).unwrap();
        assert_eq!(order, processed_order);

        let escrow_account = banks_client.get_account(escrow_key).await.unwrap().unwrap();
        assert_eq!(
            500 * 100 + Rent::default().minimum_balance(0),
            escrow_account.lamports
        );

        let sol_account = banks_client
            .get_account(order.sol_account)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(Rent::default().minimum_balance(0), sol_account.lamports);
    }
}
