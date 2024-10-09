use std::fs::File;
use std::io::{self, Write};
use crate::integer_to_letter;
use crate::declare_config;
use crate::neg_num_err;
use crate::err_print;
const MEMORY_SIZE: usize = 255;
// Add instructions here
#[derive(Debug)]
pub enum Instruction {
    ADD(usize, usize),  // (destination, source)
    MOV(usize, u16),    // (destination, immediate value)
    MUL(usize, usize),  // (destination, source)
    SUB(usize, usize),  // (destination, source)
    SWAP(usize, usize), // (reg1, reg2)
    DIV(usize, usize),
    CLR(usize), // clear one register
    INC(usize),
    DEC(usize),
    PRINT(usize),    // print a register to the console
    POW(usize, u16), // raise a value in a register to the power of something
    MOVR(usize, usize),
    HALT,
}

// CPU struct
pub struct CPU {
    pub registers: [u16; 8],
    pub memory: [u16; MEMORY_SIZE],
    pub pc: usize,
    pub running: bool,
}


impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: [0; 8],
            memory: [0; MEMORY_SIZE],
            pc: 0,          // default program counter to 0
            running: false, // create new CPU when called
        }
    }

    pub fn load_program(&mut self, program: &[Instruction]) {
        // load the program into the memory of the CPU
        for (i, instruction) in program.iter().enumerate() {
            self.memory[i] = self.encode_instruction(instruction);
        }
        println!("{:?}", self.memory);
    }

    pub fn encode_instruction(&self, instruction: &Instruction) -> u16 {
    match instruction {
        Instruction::ADD(dst, src) => {
            (0x1 << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::MOV(dst, value) => {
            (0x2 << 12) | ((*dst as u16) << 8 & 0xF00) | (*value & 0xFF)
        }
        Instruction::MUL(dst, src) => {
            (0x3 << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::SUB(dst, src) => {
            (0x4 << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::SWAP(dst, src) => {
            (0x5 << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::DIV(dst, src) => {
            (0x6 << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::CLR(src) => {
            (0x7 << 12) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::INC(src) => {
            // INC: 0x8
            (0x8 << 12) | ((*src as u16) << 4)
        }
        Instruction::DEC(src) => {
            (0x9 << 12) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::PRINT(src) => {
            (0xa << 12) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::POW(dst, value) => {
            (0xb << 12) | ((*dst as u16) << 8 & 0xF00) | (*value & 0xFF)
        }
        Instruction::MOVR(dst, src) => {
            (0xc << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0)
        }
        Instruction::HALT => {
            0x0000
        }
    }
}

    pub fn fetch_instruction(&mut self) -> u16 {
        let config = declare_config();
        let instruction = self.memory[self.pc]; // each instruction is stored one after another in the "stack"
        self.pc += 1; // add to program counter
        if config.verbose_debug {
            println!("Program Counter:\n{:?}", self.pc);
            println!("Instruction:\n{:?}", instruction);
        }
        instruction
    }

    pub fn get_register(&self, index: usize) -> Option<u16> {
        if index < self.registers.len() {
            Some(self.registers[index])
        } else {
            None // return None if index is out of bounds
        }
    }

    pub fn print_register(&self, index: usize) {
        // this is so lazy
        match self.get_register(index) {
            Some(value) => println!("{}x: {}", integer_to_letter(index), value),
            None => println!("Register index {} is out of bounds.", index),
        }
    }
    pub fn execute_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12; // extract the opcode
        let reg1 = ((instruction >> 8) & 0xF) as usize; // first register
        let reg2 = ((instruction >> 4) & 0xF) as usize; // second register
        let value = instruction & 0xFF; // value for MOV instruction
                                                 // match the opcode as hexadecimal values
        match opcode {
            0x1 => {
                // parse as 4XXX
                // ADD
                self.registers[reg1] += self.registers[reg2];
            }
            0x2 => {

                // MOV
                self.registers[reg1] = value;
            }
            0x3 => {
                // MUL
                self.registers[reg1] *= self.registers[reg2];
            }
            0x4 => {
                // SUB
                if self.registers[reg1] >= self.registers[reg2] {
                    self.registers[reg1] -= self.registers[reg2];
                } else {
                    neg_num_err("SUB");
                }
            }
            0x5 => {
                // SWAP
                self.registers.swap(reg1, reg2);
            }
            0x6 => {
                // DIVIDE (DIV)
                if self.registers[reg2] != 0 {
                    // check reg2 to avoid division by zero
                    self.registers[reg1] /= self.registers[reg2];
                } else {
                    self.running = false;
                    err_print("Dividing by zero is not allowed.".to_string());
                }
            }
            0x7 => {
                // CLR
                self.registers[reg2] = 0; // clear register
            }
            0x8 => {
                // INC
                self.registers[reg2] += 1; // add one to register provided
            }
            0x9 => {
                // DEC
                if self.registers[reg2] >= 1 {
                    self.registers[reg2] -= 1;
                } else {
                    neg_num_err("SUB");
                }
            }
            0xa => {
                // PRINT the register
                self.print_register(reg2);
            }
            0xb => {
                // POWER OF, raise first argument to the power of second
                self.registers[reg1] = u16::pow(self.registers[reg1], value.into());
            }
            0xc => {
                // MOVR, mov src into dest
                self.registers[reg1] = self.registers[reg2];
            }
            _ => {
                // halt or Invalid opcode
                self.running = false;
            }
        }
    }

    pub fn run(&mut self) {
        // run the CPU and use the previous functions
        self.running = true;
        while self.running {
            let instruction = self.fetch_instruction();
            self.execute_instruction(instruction);
        }
    }

    pub fn emit_binary(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for &instruction in &self.memory {
            // write each instruction as two bytes
            file.write_all(&instruction.to_le_bytes())?;
        }
        Ok(())
    }
}