use anchor_lang::{
    prelude::{rent, Pubkey},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        system_instruction, system_program,
    },
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::spl_token::{instruction as token_instruction, ID as TOKEN_PROGRAM_ID},
};
use litesvm::LiteSVM;
use anchor_spl::token::spl_token::state::Mint as TokenMint;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;


use amm_contract::{accounts, instruction, ID as PROGRAM_ID};

fn create_associated_token_account_instruction(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: anchor_spl::associated_token::ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(get_associated_token_address(owner, mint), false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![],
    }
}

#[test]
fn test_amm_workflow() {
    let mut svm = LiteSVM::new();

   
    svm.add_program_from_file(PROGRAM_ID, "../../target/deploy/amm_contract.so")
        .expect("Failed to load program. Did you run anchor build?");
    
    let admin = Keypair::new();
    let alice = Keypair::new(); 
    let bob = Keypair::new();   
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&alice.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&bob.pubkey(), 10_000_000_000).unwrap();

    let mint_a_kp = Keypair::new();
    let mint_b_kp = Keypair::new();
    

    let (mint_a, mint_b, _mint_a_kp, _mint_b_kp) = if mint_a_kp.pubkey() < mint_b_kp.pubkey() {
        (mint_a_kp.pubkey(), mint_b_kp.pubkey(), mint_a_kp, mint_b_kp)
    } else {
        (mint_b_kp.pubkey(), mint_a_kp.pubkey(), mint_b_kp, mint_a_kp)
    };

   
    let setup_mints_ixs = vec![
        system_instruction::create_account(&admin.pubkey(), &mint_a, 1_000_000_000, TokenMint::LEN as u64, &TOKEN_PROGRAM_ID),
        token_instruction::initialize_mint(&TOKEN_PROGRAM_ID, &mint_a, &admin.pubkey(), None, 6).unwrap(),
        system_instruction::create_account(&admin.pubkey(), &mint_b, 1_000_000_000, TokenMint::LEN as u64, &TOKEN_PROGRAM_ID),
        token_instruction::initialize_mint(&TOKEN_PROGRAM_ID, &mint_b, &admin.pubkey(), None, 6).unwrap(),
    ];

    let blockhash = svm.latest_blockhash();
    let setup_msg = Message::new_with_blockhash(&setup_mints_ixs, Some(&admin.pubkey()), &blockhash);
    let setup_tx = VersionedTransaction::try_new(VersionedMessage::Legacy(setup_msg), &[&admin, &_mint_a_kp, &_mint_b_kp]).unwrap();
    svm.send_transaction(setup_tx).unwrap();

   
    let (pool_pda, _) = Pubkey::find_program_address(&[b"amm_pool", mint_a.as_ref(), mint_b.as_ref()], &PROGRAM_ID);
    let (lp_mint_pda, _) = Pubkey::find_program_address(&[b"lp_mint", pool_pda.as_ref()], &PROGRAM_ID);
    let (vault_a_pda, _) = Pubkey::find_program_address(&[b"vault_a", pool_pda.as_ref()], &PROGRAM_ID);
    let (vault_b_pda, _) = Pubkey::find_program_address(&[b"vault_b", pool_pda.as_ref()], &PROGRAM_ID);

   
    let init_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::Initialize {
            creator: admin.pubkey(),
            mint_a,
            mint_b,
            pool: pool_pda,
            lp_mint: lp_mint_pda,
            vault_a: vault_a_pda,
            vault_b: vault_b_pda,
            system_program: system_program::ID,
            token_program: TOKEN_PROGRAM_ID,
            rent: rent::ID,
        }.to_account_metas(None),
        data: instruction::Initialize { fee_points: 30 }.data(), 
    };
    
    let blockhash = svm.latest_blockhash();
    let init_msg = Message::new_with_blockhash(&[init_ix], Some(&admin.pubkey()), &blockhash);
    let init_tx = VersionedTransaction::try_new(VersionedMessage::Legacy(init_msg), &[&admin]).unwrap();
    svm.send_transaction(init_tx).unwrap();

    
    let alice_ata_a = get_associated_token_address(&alice.pubkey(), &mint_a);
    let alice_ata_b = get_associated_token_address(&alice.pubkey(), &mint_b);
    let alice_lp_ata = get_associated_token_address(&alice.pubkey(), &lp_mint_pda);

    let setup_alice_ixs = vec![
        create_associated_token_account_instruction(&alice.pubkey(), &alice.pubkey(), &mint_a),
        create_associated_token_account_instruction(&alice.pubkey(), &alice.pubkey(), &mint_b),
        create_associated_token_account_instruction(&alice.pubkey(), &alice.pubkey(), &lp_mint_pda),
        token_instruction::mint_to(&TOKEN_PROGRAM_ID, &mint_a, &alice_ata_a, &admin.pubkey(), &[], 100_000_000).unwrap(),
        token_instruction::mint_to(&TOKEN_PROGRAM_ID, &mint_b, &alice_ata_b, &admin.pubkey(), &[], 100_000_000).unwrap(),
    ];

    let blockhash = svm.latest_blockhash();
    let alice_msg = Message::new_with_blockhash(&setup_alice_ixs, Some(&alice.pubkey()), &blockhash);
    svm.send_transaction(VersionedTransaction::try_new(VersionedMessage::Legacy(alice_msg), &[&alice, &admin]).unwrap()).unwrap();

    let deposit_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::Deposit {
            depositor: alice.pubkey(),
            pool: pool_pda,
            lp_mint: lp_mint_pda,
            vault_a: vault_a_pda,
            vault_b: vault_b_pda,
            depositor_ata_a: alice_ata_a,
            depositor_ata_b: alice_ata_b,
            depositor_lp_ata: alice_lp_ata,
            token_program: TOKEN_PROGRAM_ID,
        }.to_account_metas(None),
        data: instruction::Deposit { amount_a: 50_000_000, amount_b: 50_000_000 }.data(),
    };

    let blockhash = svm.latest_blockhash();
    let deposit_msg = Message::new_with_blockhash(&[deposit_ix], Some(&alice.pubkey()), &blockhash);
    let result = svm.send_transaction(VersionedTransaction::try_new(VersionedMessage::Legacy(deposit_msg), &[&alice]).unwrap());
    assert!(result.is_ok(), "Deposit failed: {:?}", result.err());

    
    let bob_ata_a = get_associated_token_address(&bob.pubkey(), &mint_a);
    let bob_ata_b = get_associated_token_address(&bob.pubkey(), &mint_b);

    let setup_bob_ixs = vec![
        create_associated_token_account_instruction(&bob.pubkey(), &bob.pubkey(), &mint_a),
        create_associated_token_account_instruction(&bob.pubkey(), &bob.pubkey(), &mint_b),
        token_instruction::mint_to(&TOKEN_PROGRAM_ID, &mint_a, &bob_ata_a, &admin.pubkey(), &[], 10_000_000).unwrap(),
    ];

    let blockhash = svm.latest_blockhash();
    let bob_msg = Message::new_with_blockhash(&setup_bob_ixs, Some(&bob.pubkey()), &blockhash);
    svm.send_transaction(VersionedTransaction::try_new(VersionedMessage::Legacy(bob_msg), &[&bob, &admin]).unwrap()).unwrap();

    let swap_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::Swap {
            creator: bob.pubkey(), 
            pool: pool_pda,
            vault_in: vault_a_pda,
            vault_out: vault_b_pda,
            creator_ata_in: bob_ata_a,
            creator_ata_out: bob_ata_b,
            token_program: TOKEN_PROGRAM_ID,
        }.to_account_metas(None),
        data: instruction::Swap { amount_in: 5_000_000, min_amount_out: 1 }.data(),
    };

    let blockhash = svm.latest_blockhash();
    let swap_msg = Message::new_with_blockhash(&[swap_ix], Some(&bob.pubkey()), &blockhash);
    let result = svm.send_transaction(VersionedTransaction::try_new(VersionedMessage::Legacy(swap_msg), &[&bob]).unwrap());
    assert!(result.is_ok(), "Swap failed: {:?}", result.err());

    
    let withdraw_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: accounts::Withdraw {
            withdrawer: alice.pubkey(),
            pool: pool_pda,
            lp_mint: lp_mint_pda,
            vault_a: vault_a_pda,
            vault_b: vault_b_pda,
            withdrawer_ata_a: alice_ata_a,
            withdrawer_ata_b: alice_ata_b,
            withdrawer_lp_ata: alice_lp_ata,
            token_program: TOKEN_PROGRAM_ID,
        }.to_account_metas(None),
       
        data: instruction::Withdraw { lp_amount: 10_000_000, min_amount: 0 }.data(),
    };

    let blockhash = svm.latest_blockhash();
    let withdraw_msg = Message::new_with_blockhash(&[withdraw_ix], Some(&alice.pubkey()), &blockhash);
    let result = svm.send_transaction(VersionedTransaction::try_new(VersionedMessage::Legacy(withdraw_msg), &[&alice]).unwrap());
    assert!(result.is_ok(), "Withdraw failed: {:?}", result.err());
}