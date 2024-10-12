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

// Add instructions here
#[derive(Debug)]
pub enum Instruction {
    ADD(u16, u16),
    MOV(u16, u16),
    MUL(u16, u16),
    SUB(u16, u16),
    SWAP(u16, u16),
    DIV(u16, u16),
    CLR(u16),
    INC(u16),
    DEC(u16),
    PRINT(u16),
    POW(u16, u16),
    MOVR(u16, u16),
    CMP(u16, u16),
    JMP(u16),
    HALT,
}
