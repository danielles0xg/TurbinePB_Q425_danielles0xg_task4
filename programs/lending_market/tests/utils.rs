use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// Constants
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);

use lending_market;

// Test Utils
// create Anchor instruction discriminator
pub fn anchor_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut hasher = solana_sdk::hash::Hasher::default();
    hasher.hash(preimage.as_bytes());
    let hash = hasher.result();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash.to_bytes()[..8]);
    discriminator
}

/// Helper function
pub fn create_mint(svm: &mut LiteSVM, authority: &Pubkey, decimals: u8) -> Pubkey {
    let mint_keypair = Keypair::new();
    let mint_len = 82; // Mint account size
    let rent = svm.minimum_balance_for_rent_exemption(mint_len);
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS_PER_SOL).unwrap();

    // Create account instruction manually
    let mut create_account_data = Vec::new();
    create_account_data.extend_from_slice(&[0, 0, 0, 0]); // CreateAccount discriminator
    create_account_data.extend_from_slice(&rent.to_le_bytes());
    create_account_data.extend_from_slice(&(mint_len as u64).to_le_bytes());
    create_account_data.extend_from_slice(&spl_token::id().to_bytes());

    let create_account_ix = Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint_keypair.pubkey(), true),
        ],
        data: create_account_data,
    };

    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        authority,
        None,
        decimals,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).unwrap();
    mint_keypair.pubkey()
}

// Helper to create token accounts
pub fn create_token_account(svm: &mut LiteSVM, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let token_account = Keypair::new();
    let token_account_len = 165; // Token account size
    let rent = svm.minimum_balance_for_rent_exemption(token_account_len);
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS_PER_SOL).unwrap();

    // Create account instruction manually
    let mut create_account_data = Vec::new();
    create_account_data.extend_from_slice(&[0, 0, 0, 0]); // CreateAccount discriminator
    create_account_data.extend_from_slice(&rent.to_le_bytes());
    create_account_data.extend_from_slice(&(token_account_len as u64).to_le_bytes());
    create_account_data.extend_from_slice(&spl_token::id().to_bytes());

    let create_account_ix = Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(token_account.pubkey(), true),
        ],
        data: create_account_data,
    };

    let init_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &token_account.pubkey(),
        mint,
        owner,
    )
    .unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_account_ix],
        Some(&payer.pubkey()),
        &[&payer, &token_account],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).unwrap();
    token_account.pubkey()
}

// Helper to mint tokens
pub fn mint_tokens(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    to: &Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        to,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), LAMPORTS_PER_SOL).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer, authority],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).unwrap();
}

// Helper to get token balance
pub fn get_token_balance(svm: &LiteSVM, token_account: &Pubkey) -> u64 {
    let account = svm.get_account(token_account).unwrap();
    let account_data = account.data.as_slice();
    // Token amount is stored at offset 64 in the token account data
    u64::from_le_bytes(account_data[64..72].try_into().unwrap())
}

// PDA derivation functions
pub fn get_pda_lending_market() -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(&[b"lending_market"], &program_id)
}

pub fn get_pda_asset_pair_market(loan_mint: &Pubkey, collateral_mint: &Pubkey) -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(
        &[b"asset_pair", loan_mint.as_ref(), collateral_mint.as_ref()],
        &program_id,
    )
}

pub fn get_pda_lending_offer(lender: &Pubkey, offer_id: u64) -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(
        &[b"lending_offer", lender.as_ref(), &offer_id.to_le_bytes()],
        &program_id,
    )
}

pub fn get_pda_escrow(lending_offer: &Pubkey) -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(&[b"escrow", lending_offer.as_ref()], &program_id)
}

pub fn get_pda_loan(lending_offer: &Pubkey, borrower: &Pubkey) -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(
        &[b"loan", lending_offer.as_ref(), borrower.as_ref()],
        &program_id,
    )
}

pub fn get_pda_collateral_vault(loan: &Pubkey) -> (Pubkey, u8) {
    let program_id = Pubkey::new_from_array(lending_market::ID.to_bytes());
    Pubkey::find_program_address(&[b"collateral", loan.as_ref()], &program_id)
}
