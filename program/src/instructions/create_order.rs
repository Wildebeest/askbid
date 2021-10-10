use super::{SearchMarketAccount, SearchMarketInstruction};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent, Sysvar},
};

pub fn create_order_instruction(
    program_id: &Pubkey,
    order: &Pubkey,
    search_market: &Pubkey,
    result: &Pubkey,
    proceeds_account: &Pubkey,
    token_account: &Pubkey,
    token_mint_account: &Pubkey,
    token_authority_account: &Pubkey,
    side: OrderSide,
    price: u64,
    quantity: u64,
) -> Result<Instruction, std::io::Error> {
    let (escrow_key, bump_seed) =
        Pubkey::find_program_address(&[b"token_escrow", &order.to_bytes()], program_id);
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
        AccountMeta::new(*proceeds_account, is_buy_side),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*token_mint_account, false),
        AccountMeta::new_readonly(*token_authority_account, !is_buy_side),
        AccountMeta::new(escrow_key, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct OrderAccount {
    pub search_market: Pubkey,
    pub result: Pubkey,
    pub proceeds_account: Pubkey,
    pub token_account: Pubkey,
    pub side: OrderSide,
    pub price: u64,
    pub quantity: u64,
    pub escrow_bump_seed: u8,
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
    let proceeds_account_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let token_mint_account_info = next_account_info(account_info_iter)?;
    let token_authority_account_info = next_account_info(account_info_iter)?;
    let escrow_account_info = next_account_info(account_info_iter)?;
    let spl_token_program_info = next_account_info(account_info_iter)?;
    let rent_account_info = next_account_info(account_info_iter)?;
    let clock = Clock::get()?;

    if *spl_token_program_info.key != spl_token::id() {
        return Err(ProgramError::InvalidArgument);
    }

    if *rent_account_info.key != rent::id() {
        return Err(ProgramError::InvalidArgument);
    }

    if side == OrderSide::Sell {
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

    let order = OrderAccount {
        search_market: *market_account_info.key,
        result: *result_account_info.key,
        proceeds_account: *proceeds_account_info.key,
        token_account: *token_account_info.key,
        side,
        price,
        quantity,
        escrow_bump_seed,
    };

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
    async fn test_create_order_sell() {
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

        let order_key = Pubkey::new_unique();
        let (escrow_key, bump_seed) =
            Pubkey::find_program_address(&[b"token_escrow", &order_key.to_bytes()], &program_id);
        let escrow_account = SolanaAccount::new(
            Rent::default().minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        program_test.add_account(escrow_key, escrow_account);
        let order = OrderAccount {
            search_market: market_key,
            result: result_key,
            proceeds_account: deposit_keypair.pubkey(),
            token_account: yes_token_pubkey,
            side: OrderSide::Sell,
            price: 500,
            quantity: 100,
            escrow_bump_seed: bump_seed,
        };
        let order_space = space(&order).unwrap();
        let order_min_balance = minimum_balance(&order).unwrap();
        let order_account = SolanaAccount::new(order_min_balance, order_space, &program_id);
        program_test.add_account(order_key, order_account);
        let create_order = create_order_instruction(
            &program_id,
            &order_key,
            &market_key,
            &result_key,
            &deposit_keypair.pubkey(),
            &yes_token_pubkey,
            &result.yes_mint,
            &deposit_keypair.pubkey(),
            OrderSide::Sell,
            500,
            100,
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
}
