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
        AccountMeta::new_readonly(rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
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
    let clock = Clock::get()?;

    if !result_account_info.data.borrow().iter().all(|&b| b == 0) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if *market_account_info.owner != *program_id {
        return Err(ProgramError::InvalidAccountData);
    }
    let market = SearchMarketAccount::try_from_slice(*market_account_info.data.borrow())?;
    if market.expires_slot < clock.slot {
        return Err(ProgramError::InvalidAccountData);
    }

    if url::Url::parse(&url).is_err() {
        return Err(ProgramError::InvalidArgument);
    }

    if Pubkey::create_program_address(&[b"mint_authority", &[bump_seed]], program_id)?
        != *mint_authority_info.key
    {
        return Err(ProgramError::InvalidArgument);
    }

    if *rent_account_info.key != rent::id() {
        return Err(ProgramError::InvalidAccountData);
    }

    if *spl_token_account_info.key != spl_token::id() {
        return Err(ProgramError::InvalidAccountData);
    }

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
        .map(|_| ())
        .map_err(|e| e.into())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::instructions::test_utils::setup_market;
    use crate::process_instruction;
    use crate::test_utils::*;
    use crate::undecided_result;
    use solana_program::program_pack::Pack;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::Account as SolanaAccount,
        rent::Rent,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use spl_token::state::Mint;

    pub fn setup_result(
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
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction =
            Transaction::new_with_payer(&[create_market, create_result], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &decision_authority], recent_blockhash);
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
}
