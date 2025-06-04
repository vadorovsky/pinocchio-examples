#![no_std]

use core::mem;

use pinocchio::{
    account_info::AccountInfo,
    entrypoint::{InstructionContext, MaybeAccount},
    lazy_program_entrypoint, no_allocator, nostd_panic_handler,
    program_error::ProgramError,
    pubkey::{create_program_address, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pinocchio_pubkey::declare_id!("9YxC88EDFbs4a2ypUmKy8HPUFdg1FTnwnZm7358J3w9u");

pub const COUNTER_SEED: &'static str = "counter";

/// On-chain representation of a counter.
#[derive(Clone)]
#[repr(C)]
pub struct Counter {
    pub owner: Pubkey,
    pub count: u64,
}

impl Counter {
    pub const LEN: usize = mem::size_of::<Self>();
}

/// Counter program instruction discriminators.
#[repr(u8)]
pub enum CounterInstruction {
    /// Creates/initializes a counter account for the given user.
    Create,
    /// Increments a counter.
    Increment,
    /// Decrements a counter.
    Decrement,
    /// Deletes/closes a counter account.
    Delete,
}

impl TryFrom<&u8> for CounterInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(Self::Create),
            1 => Ok(Self::Increment),
            2 => Ok(Self::Decrement),
            3 => Ok(Self::Delete),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Counter program instruction data.
#[repr(C)]
pub struct CounterInstructionData {
    pub bump: u8,
}

/// Entrypoint of the program.
pub fn process_instruction(mut context: InstructionContext) -> ProgramResult {
    // The first account is the owner of the counter.
    // If a counter is created, that account is set as an owner.
    // For all other actions, we check if the owner matches the selected
    // counter.
    let MaybeAccount::Account(mut owner) = context.next_account()? else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    // Check if the owner signed the transaction.
    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // The second account is the counter PDA.
    let MaybeAccount::Account(mut counter) = context.next_account()? else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // The third (and last) account is the system program.
    context.next_account()?;

    // Deserialize instruction and instruction data.
    let (instruction, instruction_data) = context
        .instruction_data()?
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let instruction = CounterInstruction::try_from(instruction)?;
    let instruction_data: &CounterInstructionData = unsafe { &*instruction_data.as_ptr().cast() };

    let counter_pda = create_program_address(
        &[
            COUNTER_SEED.as_bytes(),
            owner.key(),
            &[instruction_data.bump],
        ],
        &ID,
    )?;
    if counter.key() != &counter_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    match instruction {
        CounterInstruction::Create => process_create(&owner, &mut counter)?,
        CounterInstruction::Increment => process_increment(&owner, &mut counter)?,
        CounterInstruction::Decrement => process_decrement(&owner, &mut counter)?,
        CounterInstruction::Delete => process_delete(&mut owner, &mut counter)?,
    }

    Ok(())
}

/// Creates/initializes a counter account for the given user.
pub fn process_create(owner: &AccountInfo, counter: &mut AccountInfo) -> ProgramResult {
    // Create the PDA.
    CreateAccount {
        from: owner,
        to: &counter,
        lamports: Rent::get()?.minimum_balance(Counter::LEN),
        space: Counter::LEN as u64,
        owner: &ID,
    }
    .invoke()?;

    // Deserialize the counter PDA.
    let mut data = counter.try_borrow_mut_data()?;
    let data: &mut Counter = unsafe { &mut *data.as_mut_ptr().cast() };

    // Initialize the counter.
    data.owner = *owner.key();
    data.count = 0;

    log!("Created the counter account");

    Ok(())
}

/// Increments a counter.
pub fn process_increment(owner: &AccountInfo, counter: &mut AccountInfo) -> ProgramResult {
    // Check if the counter PDA is owned by the program.
    if !counter.is_owned_by(&ID) {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize the counter PDA.
    let mut data = counter.try_borrow_mut_data()?;
    let data: &mut Counter = unsafe { &mut *data.as_mut_ptr().cast() };

    // Check if the counter was created by the `owner`.
    if &data.owner != owner.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Increment the counter.
    data.count = data.count.saturating_add(1);

    log!("Incremented the counter to {}", data.count);

    Ok(())
}

/// Deletes/closes a counter account.
pub fn process_decrement(owner: &AccountInfo, counter: &mut AccountInfo) -> ProgramResult {
    // Check if the counter PDA is owned by the program.
    if !counter.is_owned_by(&ID) {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize the counter PDA.
    let mut data = counter.try_borrow_mut_data()?;
    let data: &mut Counter = unsafe { &mut *data.as_mut_ptr().cast() };

    // Check if the counter has correct ownership.
    if &data.owner != owner.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Decrement the counter.
    data.count = data.count.saturating_sub(1);

    log!("Decremented the counter to {}", data.count);

    Ok(())
}

/// Decrements a counter.
pub fn process_delete(owner: &mut AccountInfo, counter: &mut AccountInfo) -> ProgramResult {
    // Check if the counter PDA is owned by the program.
    if !counter.is_owned_by(&ID) {
        return Err(ProgramError::IllegalOwner);
    }

    // Deserialize the counter PDA.
    let mut data = counter.try_borrow_mut_data()?;
    let data: &mut Counter = unsafe { &mut *data.as_mut_ptr().cast() };

    // Check if the counter has correct ownership.
    if &data.owner != owner.key() {
        return Err(ProgramError::IllegalOwner);
    }

    // Close the counter account by moving its lamports to the owner.
    let mut owner_lamports = owner.try_borrow_mut_lamports()?;
    let mut counter_lamports = counter.try_borrow_mut_lamports()?;
    *owner_lamports = owner_lamports.saturating_add(*counter_lamports);
    *counter_lamports = 0;

    Ok(())
}
