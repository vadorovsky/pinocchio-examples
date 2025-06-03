#![no_std]

use pinocchio::{
    entrypoint::InstructionContext, lazy_program_entrypoint, no_allocator, nostd_panic_handler,
    ProgramResult,
};
use pinocchio_log::log;

lazy_program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pinocchio_pubkey::declare_id!("CYfPbdyLefX3mmAQJfiarrUWjERYLS7iTTqeGTgoxWr2");

pub fn process_instruction(_context: InstructionContext) -> ProgramResult {
    log!("Hello, world!");
    Ok(())
}
