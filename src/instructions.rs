pub const MEMORY_SIZE: usize = 255;

// Opcode constants
pub const ADD_OPCODE: u16 = 0x1;
pub const MOV_OPCODE: u16 = 0x2;
pub const MUL_OPCODE: u16 = 0x3;
pub const SUB_OPCODE: u16 = 0x4;
pub const SWAP_OPCODE: u16 = 0x5;
pub const DIV_OPCODE: u16 = 0x6;
pub const CLR_OPCODE: u16 = 0x7;
pub const INC_OPCODE: u16 = 0x8;
pub const DEC_OPCODE: u16 = 0x9;
pub const PRINT_OPCODE: u16 = 0xa;
pub const POW_OPCODE: u16 = 0xb;
pub const MOVR_OPCODE: u16 = 0xc;
pub const CMP_OPCODE: u16 = 0xd;
pub const JMP_OPCODE: u16 = 0xe;
pub const HALT_OPCODE: u16 = 0x0;

// bro add call you absolute babooon

// Add instructions here
#[derive(Debug)]
pub enum Instruction {
    ADD(usize, usize),
    MOV(usize, u16),
    MUL(usize, usize),
    SUB(usize, usize),
    SWAP(usize, usize),
    DIV(usize, usize),
    CLR(usize),
    INC(usize),
    DEC(usize),
    PRINT(usize),
    POW(usize, u16),
    MOVR(usize, usize),
    CMP(usize, usize),
    JMP(usize),
    HALT,
}
