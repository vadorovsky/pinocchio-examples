use mollusk_svm::{
    result::{Check, ProgramResult},
    Mollusk,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

const ID: Pubkey = Pubkey::new_from_array(hello_world::ID);

#[test]
fn test_hello_world() {
    let mollusk = Mollusk::new(&ID, "target/deploy/hello_world");

    let data = &[];
    let ix_accounts = Vec::new();
    let tx_accounts = &[];
    let res = mollusk.process_and_validate_instruction(
        &Instruction::new_with_bytes(ID, data, ix_accounts),
        tx_accounts,
        &[Check::success()],
    );
    assert!(matches!(res.program_result, ProgramResult::Success));
}
