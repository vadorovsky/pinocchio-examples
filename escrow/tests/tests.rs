use std::mem;

use escrow::{
    Escrow, EscrowInstruction, FinalizeInstructionData, InitializeInstructionData, ESCROW_SEED,
};
use mollusk_svm::{
    program::{
        create_program_account_loader_v3, keyed_account_for_system_program, loader_keys::LOADER_V3,
    },
    result::{Check, ProgramResult},
    Mollusk,
};
use solana_account::{Account, WritableAccount};
use solana_instruction::{AccountMeta, Instruction};
use solana_native_token::LAMPORTS_PER_SOL;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_token::{
    solana_program::program_option::COption,
    state::{Account as TokenAccount, AccountState as TokenAccountState, Mint},
};

const ID: Pubkey = Pubkey::new_from_array(escrow::ID);
const TOKEN_ID: Pubkey = Pubkey::new_from_array(pinocchio_token::ID);

fn instruction_initialize(
    amount: u64,
    sender: &Pubkey,
    sender_ata: &Pubkey,
    receiver: &Pubkey,
    escrow: &Pubkey,
    escrow_ata: &Pubkey,
    bump: u8,
    system_program: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    // Create instruction data.
    let data = InitializeInstructionData::new(amount, bump);
    // Serialize instruction data to bytes.
    let data = unsafe {
        &*(&data as *const InitializeInstructionData
            as *const [u8; size_of::<InitializeInstructionData>()])
    };

    // Construct the full instruction data, consisting of:
    // * discriminator
    // * serialized data
    let mut data_with_discriminator: Vec<u8> = Vec::with_capacity(
        mem::size_of::<EscrowInstruction>() + mem::size_of::<InitializeInstructionData>(),
    );
    data_with_discriminator.push(EscrowInstruction::Initialize as u8);
    data_with_discriminator.extend_from_slice(data);

    let ix_accounts = vec![
        AccountMeta::new(*sender, true),
        AccountMeta::new(*sender_ata, false),
        AccountMeta::new(*receiver, false),
        AccountMeta::new(*escrow, true),
        AccountMeta::new(*escrow_ata, false),
        AccountMeta::new_readonly(*system_program, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction::new_with_bytes(ID, &data_with_discriminator, ix_accounts)
}

fn instruction_exchange(
    receiver: &Pubkey,
    receiver_ata: &Pubkey,
    escrow: &Pubkey,
    escrow_ata: &Pubkey,
    bump: u8,
    system_program: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    // Create instruction data.
    let data = FinalizeInstructionData::new(bump);
    // Serialize instruction data to bytes.
    let data = unsafe {
        &*(&data as *const FinalizeInstructionData
            as *const [u8; size_of::<FinalizeInstructionData>()])
    };

    // Construct the full instruction data, consisting of:
    // * discriminator
    // * serialized data
    let mut data_with_discriminator: Vec<u8> = Vec::with_capacity(
        mem::size_of::<EscrowInstruction>() + mem::size_of::<FinalizeInstructionData>(),
    );
    data_with_discriminator.push(EscrowInstruction::Exchange as u8);
    data_with_discriminator.extend_from_slice(data);

    let ix_accounts = vec![
        AccountMeta::new(*receiver, true),
        AccountMeta::new(*receiver_ata, false),
        AccountMeta::new(*escrow, true),
        AccountMeta::new(*escrow_ata, false),
        AccountMeta::new_readonly(*system_program, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction::new_with_bytes(ID, &data_with_discriminator, ix_accounts)
}

fn instruction_cancel(
    sender: &Pubkey,
    sender_ata: &Pubkey,
    escrow: &Pubkey,
    escrow_ata: &Pubkey,
    bump: u8,
    system_program: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    // Create instruction data.
    let data = FinalizeInstructionData::new(bump);
    // Serialize instruction data to bytes.
    let data = unsafe {
        &*(&data as *const FinalizeInstructionData
            as *const [u8; size_of::<FinalizeInstructionData>()])
    };

    // Construct the full instruction data, consisting of:
    // * discriminator
    // * serialized data
    let mut data_with_discriminator: Vec<u8> = Vec::with_capacity(
        mem::size_of::<EscrowInstruction>() + mem::size_of::<FinalizeInstructionData>(),
    );
    data_with_discriminator.push(EscrowInstruction::Cancel as u8);
    data_with_discriminator.extend_from_slice(data);

    let ix_accounts = vec![
        AccountMeta::new(*sender, true),
        AccountMeta::new(*sender_ata, false),
        AccountMeta::new(*escrow, true),
        AccountMeta::new(*escrow_ata, false),
        AccountMeta::new_readonly(*system_program, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction::new_with_bytes(ID, &data_with_discriminator, ix_accounts)
}

#[test]
fn test_escrow_initialize_success() {
    let mut mollusk = Mollusk::new(&ID, "target/deploy/escrow");
    mollusk.add_program(&TOKEN_ID, "third-party/spl_token", &LOADER_V3);

    let (system_program, system_account) = keyed_account_for_system_program();
    let (token_program, token_account) = (TOKEN_ID, create_program_account_loader_v3(&TOKEN_ID));

    // Initialize mint.
    let mint = Pubkey::new_unique();
    let mut mint_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &token_program,
    );
    Pack::pack(
        Mint {
            mint_authority: COption::None,
            supply: 1_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_account.data_as_mut_slice(),
    )
    .unwrap();

    let sender = Pubkey::new_unique();
    let sender_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let sender_ata = Pubkey::new_unique();
    let mut sender_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: sender,
            amount: 1_000_000,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        sender_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let receiver = Pubkey::new_unique();
    let receiver_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let (escrow, bump) = Pubkey::find_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.as_array(),
            receiver.as_array(),
        ],
        &ID,
    );
    // We don't specify the space for the escrow PDA yet - we are letting the
    // `create` instruction do that.
    let escrow_account = Account::new(0, 0, &system_program);

    let escrow_ata = Pubkey::new_unique();
    let mut escrow_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: escrow,
            amount: 0,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        escrow_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let tx_accounts = &[
        (sender, sender_account),
        (sender_ata, sender_ata_account),
        (receiver, receiver_account),
        (escrow, escrow_account),
        (escrow_ata, escrow_ata_account),
        (system_program, system_account),
        (token_program, token_account),
    ];
    let res = mollusk.process_and_validate_instruction_chain(
        &[(
            &instruction_initialize(
                100,
                &sender,
                &sender_ata,
                &receiver,
                &escrow,
                &escrow_ata,
                bump,
                &system_program,
                &token_program,
            ),
            &[Check::success()],
        )],
        tx_accounts,
    );
    assert!(matches!(res.program_result, ProgramResult::Success));
}

#[test]
fn test_escrow_exchange_success() {
    let mut mollusk = Mollusk::new(&ID, "target/deploy/escrow");
    mollusk.add_program(&TOKEN_ID, "third-party/spl_token", &LOADER_V3);

    let (system_program, system_account) = keyed_account_for_system_program();
    let (token_program, token_account) = (TOKEN_ID, create_program_account_loader_v3(&TOKEN_ID));

    // Initialize mint.
    let mint = Pubkey::new_unique();
    let mut mint_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &token_program,
    );
    Pack::pack(
        Mint {
            mint_authority: COption::None,
            supply: 1_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_account.data_as_mut_slice(),
    )
    .unwrap();

    let sender = Pubkey::new_unique();

    let receiver = Pubkey::new_unique();
    let receiver_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let receiver_ata = Pubkey::new_unique();
    let mut receiver_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: receiver,
            amount: 0,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        receiver_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let (escrow, bump) = Pubkey::find_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.as_array(),
            receiver.as_array(),
        ],
        &ID,
    );
    // We don't specify the space for the escrow PDA yet - we are letting the
    // `create` instruction do that.
    let mut escrow_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Escrow::LEN),
        Escrow::LEN,
        &system_program,
    );
    let escrow_data = Escrow {
        sender: sender.to_bytes(),
        receiver: receiver.to_bytes(),
        amount: 100,
    };
    let escrow_data =
        unsafe { &*(&escrow_data as *const Escrow as *const [u8; size_of::<Escrow>()]) };
    escrow_account.data.copy_from_slice(escrow_data);

    let escrow_ata = Pubkey::new_unique();
    let mut escrow_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: escrow,
            amount: 100,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        escrow_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let tx_accounts = &[
        (receiver, receiver_account),
        (receiver_ata, receiver_ata_account),
        (escrow, escrow_account),
        (escrow_ata, escrow_ata_account),
        (system_program, system_account),
        (token_program, token_account),
    ];
    let res = mollusk.process_and_validate_instruction_chain(
        &[(
            &instruction_exchange(
                &receiver,
                &receiver_ata,
                &escrow,
                &escrow_ata,
                bump,
                &system_program,
                &token_program,
            ),
            &[Check::success()],
        )],
        tx_accounts,
    );
    assert!(matches!(res.program_result, ProgramResult::Success));
}

#[test]
fn test_escrow_cancel_success() {
    let mut mollusk = Mollusk::new(&ID, "target/deploy/escrow");
    mollusk.add_program(&TOKEN_ID, "third-party/spl_token", &LOADER_V3);

    let (system_program, system_account) = keyed_account_for_system_program();
    let (token_program, token_account) = (TOKEN_ID, create_program_account_loader_v3(&TOKEN_ID));

    // Initialize mint.
    let mint = Pubkey::new_unique();
    let mut mint_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Mint::LEN),
        Mint::LEN,
        &token_program,
    );
    Pack::pack(
        Mint {
            mint_authority: COption::None,
            supply: 1_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        },
        mint_account.data_as_mut_slice(),
    )
    .unwrap();

    let sender = Pubkey::new_unique();
    let sender_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let sender_ata = Pubkey::new_unique();
    let mut sender_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: sender,
            amount: 1_000_000,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        sender_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let receiver = Pubkey::new_unique();

    let (escrow, bump) = Pubkey::find_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.as_array(),
            receiver.as_array(),
        ],
        &ID,
    );
    let mut escrow_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(Escrow::LEN),
        Escrow::LEN,
        &system_program,
    );
    let escrow_data = Escrow {
        sender: sender.to_bytes(),
        receiver: receiver.to_bytes(),
        amount: 100,
    };
    let escrow_data =
        unsafe { &*(&escrow_data as *const Escrow as *const [u8; size_of::<Escrow>()]) };
    escrow_account.data.copy_from_slice(escrow_data);

    let escrow_ata = Pubkey::new_unique();
    let mut escrow_ata_account = Account::new(
        mollusk.sysvars.rent.minimum_balance(TokenAccount::LEN),
        TokenAccount::LEN,
        &token_program,
    );
    Pack::pack(
        TokenAccount {
            mint,
            owner: escrow,
            amount: 100,
            delegate: COption::None,
            state: TokenAccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        escrow_ata_account.data_as_mut_slice(),
    )
    .unwrap();

    let tx_accounts = &[
        (sender, sender_account),
        (sender_ata, sender_ata_account),
        (escrow, escrow_account),
        (escrow_ata, escrow_ata_account),
        (system_program, system_account),
        (token_program, token_account),
    ];
    let res = mollusk.process_and_validate_instruction_chain(
        &[(
            &instruction_cancel(
                &sender,
                &sender_ata,
                &escrow,
                &escrow_ata,
                bump,
                &system_program,
                &token_program,
            ),
            &[Check::success()],
        )],
        tx_accounts,
    );
    assert!(matches!(res.program_result, ProgramResult::Success));
}
