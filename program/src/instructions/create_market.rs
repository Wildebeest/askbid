use super::{AccountType, SearchMarketInstruction};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Slot,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub struct SearchMarketAccount {
    pub account_type: AccountType,
    pub account_version: u8,
    pub decision_authority: Pubkey,
    pub search_string: String,
    pub best_result: Pubkey,
    pub expires_slot: Slot,
}

impl SearchMarketAccount {
    pub fn new(
        decision_authority: Pubkey,
        search_string: String,
        expires_slot: Slot,
    ) -> SearchMarketAccount {
        SearchMarketAccount {
            account_type: AccountType::SearchMarket,
            account_version: 0,
            decision_authority,
            search_string,
            expires_slot,
            best_result: Pubkey::default(),
        }
    }
}

pub fn create_market(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    expires_slot: Slot,
    search_string: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let market_account_info = next_account_info(account_info_iter)?;
    let decision_authority_info = next_account_info(account_info_iter)?;

    if !market_account_info.data.borrow().iter().all(|&b| b == 0) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !decision_authority_info.is_signer {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let search_market =
        SearchMarketAccount::new(*decision_authority_info.key, search_string, expires_slot);

    let result = search_market
        .serialize(&mut &mut market_account_info.data.borrow_mut()[..])
        .map(|_| ())
        .map_err(|e| e.into());

    return result;
}

pub fn create_market_instruction(
    program_id: &Pubkey,
    market_pubkey: &Pubkey,
    decision_pubkey: &Pubkey,
    expires_slot: Slot,
    search_string: String,
) -> Result<Instruction, std::io::Error> {
    let data = SearchMarketInstruction::CreateMarket {
        expires_slot,
        search_string,
    }
    .try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*market_pubkey, false),
        AccountMeta::new_readonly(*decision_pubkey, true),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::process_instruction;
    use crate::test_utils::*;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::Account as SolanaAccount,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };

    pub fn setup_market(
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
            &market.decision_authority,
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

        let decision_authority = Keypair::new();
        let market =
            SearchMarketAccount::new(decision_authority.pubkey(), "cyberpunk".to_string(), 0);
        let (market_key, create_market) = setup_market(&market, &mut program_test, &program_id);

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let mut transaction = Transaction::new_with_payer(&[create_market], Some(&payer.pubkey()));
        transaction.sign(&[&payer, &decision_authority], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let market_account = banks_client.get_account(market_key).await.unwrap().unwrap();
        let processed_market =
            SearchMarketAccount::try_from_slice(&market_account.data[..]).unwrap();
        assert_eq!(market, processed_market);
    }
}
