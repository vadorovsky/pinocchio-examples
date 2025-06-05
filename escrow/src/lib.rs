#![no_std]

use core::mem;

use pinocchio::{
    account_info::AccountInfo,
    no_allocator, nostd_panic_handler, program_entrypoint,
    program_error::ProgramError,
    pubkey::{create_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pinocchio_pubkey::declare_id!("AMeUviQdjAPsvfWwRfboCLrN7t2fjSxqs4eMZguezpQr");

pub const ESCROW_SEED: &'static str = "escrow";

#[derive(Clone)]
#[repr(C)]
pub struct Escrow {
    pub sender: Pubkey,
    pub receiver: Pubkey,
    pub amount: u64,
}

impl Escrow {
    pub const LEN: usize = mem::size_of::<Self>();
}

#[repr(u8)]
pub enum EscrowInstruction {
    Initialize,
    Exchange,
    Cancel,
}

impl TryFrom<&u8> for EscrowInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Exchange),
            2 => Ok(Self::Cancel),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
pub struct InitializeInstructionData {
    pub amount: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl InitializeInstructionData {
    pub fn new(amount: u64, bump: u8) -> Self {
        Self {
            amount,
            bump,
            _padding: [0; 7],
        }
    }
}

#[repr(C)]
pub struct FinalizeInstructionData {
    pub bump: u8,
}

impl FinalizeInstructionData {
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }
}

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (instruction, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let instruction = EscrowInstruction::try_from(instruction)?;

    match instruction {
        EscrowInstruction::Initialize => process_initialize(accounts, instruction_data),
        EscrowInstruction::Exchange => process_exchange(accounts, instruction_data),
        EscrowInstruction::Cancel => process_cancel(accounts, instruction_data),
    }
}

pub fn process_initialize(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Retrieve and validate the accounts.
    let [sender, sender_ata, receiver, escrow, escrow_ata, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check that `sender_ata` is owned by `sender`.
    if TokenAccount::from_account_info(sender_ata)?.owner() != sender.key() {
        return Err(ProgramError::IllegalOwner);
    }
    // Check that `escrow_ata` is owned by `escrow`.
    if TokenAccount::from_account_info(escrow_ata)?.owner() != escrow.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize instruction data.
    let instruction_data: &InitializeInstructionData =
        unsafe { &*instruction_data.as_ptr().cast() };

    // Check the seeds of `escrow`.
    let escrow_pda = create_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.key(),
            receiver.key(),
            &[instruction_data.bump],
        ],
        &ID,
    )?;
    if escrow.key() != &escrow_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Create the escrow PDA.
    CreateAccount {
        from: &sender,
        to: &escrow,
        lamports: Rent::get()?.minimum_balance(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &ID,
    }
    .invoke()?;

    // Deserialize the escrow PDA.
    let mut data = escrow.try_borrow_mut_data()?;
    let data: &mut Escrow = unsafe { &mut *data.as_mut_ptr().cast() };

    // Initialize the escrow.
    data.sender = *sender.key();
    data.receiver = *receiver.key();

    // Transfer token from sender to escrow.
    Transfer {
        from: &sender_ata,
        to: &escrow_ata,
        authority: &sender,
        amount: instruction_data.amount,
    }
    .invoke()?;

    log!("Initialized escrow");

    Ok(())
}

pub fn process_exchange(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Retrieve and validate the accounts.
    let [sender, receiver, receiver_ata, escrow, escrow_ata, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check that `receiver_ata` is owned by `receiver`.
    if TokenAccount::from_account_info(receiver_ata)?.owner() != receiver.key() {
        return Err(ProgramError::IllegalOwner);
    }
    // Check that `escrow_ata` is owned by `escrow`.
    if TokenAccount::from_account_info(escrow_ata)?.owner() != escrow.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize instruction data.
    let instruction_data: &FinalizeInstructionData = unsafe { &*instruction_data.as_ptr().cast() };

    // Check the seeds of `escrow`.
    let escrow_pda = create_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.key(),
            receiver.key(),
            &[instruction_data.bump],
        ],
        &ID,
    )?;
    if escrow.key() != &escrow_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Deserialize the escrow PDA.
    let data = escrow.try_borrow_data()?;
    let data: &Escrow = unsafe { &*data.as_ptr().cast() };

    // Check that `receiver` is the same as in the escrow account.
    if &data.receiver != receiver.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Transfer tokens from escrow to recipient.
    Transfer {
        from: &escrow_ata,
        to: &receiver_ata,
        authority: &escrow,
        amount: data.amount,
    }
    .invoke()?;

    log!("Exchanged {} tokens", data.amount);

    Ok(())
}

pub fn process_cancel(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    // Retrieve and validate the accounts.
    let [sender, sender_ata, receiver, escrow, escrow_ata, _system_program, _token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check that `sender_ata` is owned by `sender`.
    if TokenAccount::from_account_info(sender_ata)?.owner() != sender.key() {
        return Err(ProgramError::IllegalOwner);
    }
    // Check that `escrow_ata` is owned by `escrow`.
    if TokenAccount::from_account_info(escrow_ata)?.owner() != escrow.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize instruction data.
    let instruction_data: &FinalizeInstructionData = unsafe { &*instruction_data.as_ptr().cast() };

    // Check the seeds of `escrow`.
    let escrow_pda = create_program_address(
        &[
            ESCROW_SEED.as_bytes(),
            sender.key(),
            receiver.key(),
            &[instruction_data.bump],
        ],
        &ID,
    )?;
    if escrow.key() != &escrow_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let data = escrow.try_borrow_data()?;
    let data: &Escrow = unsafe { &*data.as_ptr().cast() };

    // Check that escrow was initailized by `sender`.
    if &data.sender != sender.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Transfer tokens from escrow to sender.
    Transfer {
        from: &escrow_ata,
        to: &sender_ata,
        authority: &escrow,
        amount: data.amount,
    }
    .invoke()?;

    log!("Cancelled escrow, refunded {} tokens", data.amount);

    Ok(())
}
