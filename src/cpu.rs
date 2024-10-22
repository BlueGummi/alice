use crate::*;
use std::fs::File;
use std::io::{self, Read, Write};

// CPU struct
pub struct CPU {
    pub registers: [u16; 16],
    pub memory: [u16; MEMORY_SIZE],
    pub pc: u16, // Change to u16
    pub running: bool,
    pub zflag: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: [0; 16],
            memory: [0; MEMORY_SIZE],
            pc: 0,
            running: false,
            zflag: false,
        }
    }

    pub fn load_program(&mut self, program: &[Instruction]) {
        for (i, instruction) in program.iter().enumerate() {
            if i < MEMORY_SIZE {
                self.memory[i] = self.encode_instruction(instruction);
            } else {
                eprintln!(
                    "{}",
                    "Warning: Program exceeds memory size.".color(Colors::RedFg)
                );
                break;
            }
        }
        let config = declare_config();
        if config.verbose_debug {
            println!("{:?}", self.memory);
        }
    }

    pub fn encode_instruction(&self, instruction: &Instruction) -> u16 {
        match instruction {
            Instruction::ADD(dst, src) => {
                (ADD_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::MOV(dst, value) => {
                (MOV_OPCODE << 12) | ((*dst) << 8 & 0xF00) | (*value & 0xFF)
            }
            Instruction::MUL(dst, src) => {
                (MUL_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::SUB(dst, src) => {
                (SUB_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::SWAP(dst, src) => {
                (SWAP_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::DIV(dst, src) => {
                (DIV_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::CLR(src) => (CLR_OPCODE << 12) | ((*src) << 4 & 0x0F0),
            Instruction::INC(src) => (INC_OPCODE << 12) | ((*src) << 4 & 0x0F0),
            Instruction::DEC(src) => (DEC_OPCODE << 12) | ((*src) << 4 & 0x0F0),
            Instruction::PRINT(src) => (PRINT_OPCODE << 12) | ((*src) << 4 & 0x0F0),
            Instruction::POW(dst, value) => {
                (POW_OPCODE << 12) | ((*dst) << 8 & 0xF00) | (*value & 0xFF)
            }
            Instruction::MOVR(dst, src) => {
                (MOVR_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::CMP(dst, src) => {
                (CMP_OPCODE << 12) | ((*dst) << 8 & 0xF00) | ((*src) << 4 & 0x0F0)
            }
            Instruction::JMP(src) => (JMP_OPCODE << 12) | ((*src) << 8 & 0xF00),
            Instruction::HALT => HALT_OPCODE << 12,
            Instruction::NOP => NOP_OPCODE << 12,
        }
    }

    pub fn fetch_instruction(&mut self) -> Option<u16> {
        if self.pc < MEMORY_SIZE as u16 {
            let config = declare_config();
            let instruction = self.memory[self.pc as usize];
            self.pc += 1;

            if config.verbose_debug {
                println!("Program Counter: {:?}", self.pc);
                println!("Instruction: {:?}", instruction);
            }
            Some(instruction)
        } else {
            None
        }
    }

    pub fn get_register(&self, index: u16) -> Option<u16> {
        if index < self.registers.len() as u16 {
            Some(self.registers[index as usize])
        } else {
            None
        }
    }

    pub fn print_register(&self, index: u16) {
        match self.get_register(index) {
            Some(value) => println!("{}x: {}", integer_to_letter(index as usize), value),
            None => println!("Register index {} is out of bounds.", index),
        }
    }

    pub fn execute_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        let reg1 = (instruction >> 8) & 0xF; // Change to u16
        let reg2 = (instruction >> 4) & 0xF; // Change to u16
        let value = instruction & 0xFF;

        match opcode {
            ADD_OPCODE => self.registers[reg1 as usize] += self.registers[reg2 as usize],
            MOV_OPCODE => self.registers[reg1 as usize] = value,
            MUL_OPCODE => self.registers[reg1 as usize] *= self.registers[reg2 as usize],
            SUB_OPCODE => {
                if self.registers[reg1 as usize] >= self.registers[reg2 as usize] {
                    self.registers[reg1 as usize] -= self.registers[reg2 as usize];
                } else {
                    neg_num_err("SUB");
                }
            }
            SWAP_OPCODE => self.registers.swap(reg1 as usize, reg2 as usize),
            DIV_OPCODE => {
                if self.registers[reg2 as usize] != 0 {
                    self.registers[reg1 as usize] /= self.registers[reg2 as usize];
                } else {
                    self.running = false;
                    err_print("Dividing by zero is not allowed.".to_string());
                }
            }
            CLR_OPCODE => self.registers[reg2 as usize] = 0,
            INC_OPCODE => self.registers[reg2 as usize] += 1,
            DEC_OPCODE => {
                if self.registers[reg2 as usize] >= 1 {
                    self.registers[reg2 as usize] -= 1;
                } else {
                    neg_num_err("DEC");
                }
            }
            PRINT_OPCODE => self.print_register(reg2),
            POW_OPCODE => {
                self.registers[reg1 as usize] =
                    u16::pow(self.registers[reg1 as usize], value.into())
            }
            MOVR_OPCODE => self.registers[reg1 as usize] = self.registers[reg2 as usize],
            CMP_OPCODE => {
                self.zflag = self.registers[reg1 as usize] == self.registers[reg2 as usize];
            }
            JMP_OPCODE => {
                // Here, we interpret `value` as the new program counter (PC) address
                let jump_address = value; // Ensure this is cast correctly
                self.pc = jump_address; // Update the program counter to the jump address
            }
            NOP_OPCODE => {}
            _ => self.running = false,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        while self.running {
            if let Some(instruction) = self.fetch_instruction() {
                self.execute_instruction(instruction);
            } else {
                self.running = false;
            }
        }
    }

    pub fn emit_binary(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        for &instruction in &self.memory {
            if instruction != 0 {
                file.write_all(&instruction.to_be_bytes())?;
            }
        }
        Ok(())
    }

    pub fn load_binary(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        for (i, chunk) in buffer.chunks_exact(2).enumerate() {
            if i < MEMORY_SIZE {
                let instruction = u16::from_be_bytes([chunk[0], chunk[1]]);
                self.memory[i] = instruction;
            } else {
                eprintln!("Warning: Binary exceeds memory size.");
                break;
            }
        }
        self.pc = 0;
        Ok(())
    }
}
