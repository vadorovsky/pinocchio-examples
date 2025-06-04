use std::mem;

use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, ProgramResult},
    Mollusk,
};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;

use counter::{CounterInstruction, CounterInstructionData, COUNTER_SEED};

const ID: Pubkey = Pubkey::new_from_array(counter::ID);

/// Creates a full instruction.
fn instruction(
    counter_instruction: CounterInstruction,
    owner: &Pubkey,
    counter: &Pubkey,
    bump: u8,
    system_program: &Pubkey,
) -> Instruction {
    // Create instruction data.
    let data = CounterInstructionData { bump };
    // Serialize instruction data to bytes.
    let data = unsafe {
        &*(&data as *const CounterInstructionData
            as *const [u8; size_of::<CounterInstructionData>()])
    };

    // Construct the full instruction data, consisting of:
    // * discriminator
    // * serialized data
    let mut data_with_discriminator: Vec<u8> = Vec::with_capacity(
        mem::size_of::<CounterInstruction>() + mem::size_of::<CounterInstructionData>(),
    );
    data_with_discriminator.push(counter_instruction as u8);
    data_with_discriminator.extend_from_slice(data);

    let ix_accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*counter, true),
        AccountMeta::new_readonly(*system_program, false),
    ];
    Instruction::new_with_bytes(ID, &data_with_discriminator, ix_accounts)
}

#[test]
fn test_counter_success() {
    let mollusk = Mollusk::new(&ID, "target/deploy/counter");
    let (system_program, system_account) = keyed_account_for_system_program();

    let owner = Pubkey::new_unique();
    let owner_account = Account::new(42 * LAMPORTS_PER_SOL, 0, &system_program);

    let (counter, bump) =
        Pubkey::find_program_address(&[COUNTER_SEED.as_bytes(), owner.as_array()], &ID);
    // We don't specify the space for the counter PDA yet - we are letting the
    // `create` instruction do that.
    let counter_account = Account::new(0, 0, &system_program);

    let tx_accounts = &[
        (owner, owner_account.clone()),
        (counter, counter_account.clone()),
        (system_program, system_account.clone()),
    ];
    let res = mollusk.process_and_validate_instruction_chain(
        &[
            // Create/initialize the counter.
            (
                &instruction(
                    CounterInstruction::Create,
                    &owner,
                    &counter,
                    bump,
                    &system_program,
                ),
                &[Check::success()],
            ),
            (
                &instruction(
                    CounterInstruction::Increment,
                    &owner,
                    &counter,
                    bump,
                    &system_program,
                ),
                &[Check::success()],
            ),
            (
                &instruction(
                    CounterInstruction::Decrement,
                    &owner,
                    &counter,
                    bump,
                    &system_program,
                ),
                &[Check::success()],
            ),
            // Delete/close the counter.
            (
                &instruction(
                    CounterInstruction::Delete,
                    &owner,
                    &counter,
                    bump,
                    &system_program,
                ),
                &[Check::success()],
            ),
        ],
        tx_accounts,
    );
    assert!(matches!(res.program_result, ProgramResult::Success));
}
