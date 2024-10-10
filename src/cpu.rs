use std::fs::File;
use std::io::{self, Write, Read};
use crate::integer_to_letter;
use crate::declare_config;
use crate::neg_num_err;
use crate::err_print;
use crate::instructions::*;

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
            pc: 0,
            running: false,
        }
    }

    pub fn load_program(&mut self, program: &[Instruction]) {
        for (i, instruction) in program.iter().enumerate() {
            if i < MEMORY_SIZE {
                self.memory[i] = self.encode_instruction(instruction);
            } else {
                eprintln!("Warning: Program exceeds memory size.");
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
            Instruction::ADD(dst, src) => (ADD_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::MOV(dst, value) => (MOV_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | (*value & 0xFF),
            Instruction::MUL(dst, src) => (MUL_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::SUB(dst, src) => (SUB_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::SWAP(dst, src) => (SWAP_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::DIV(dst, src) => (DIV_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::CLR(src) => (CLR_OPCODE << 12) | ((*src as u16) << 4 & 0x0F0),
            Instruction::INC(src) => (INC_OPCODE << 12) | ((*src as u16) << 4),
            Instruction::DEC(src) => (DEC_OPCODE << 12) | ((*src as u16) << 4 & 0x0F0),
            Instruction::PRINT(src) => (PRINT_OPCODE << 12) | ((*src as u16) << 4 & 0x0F0),
            Instruction::POW(dst, value) => (POW_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | (*value & 0xFF),
            Instruction::MOVR(dst, src) => (MOVR_OPCODE << 12) | ((*dst as u16) << 8 & 0xF00) | ((*src as u16) << 4 & 0x0F0),
            Instruction::HALT => HALT_OPCODE << 12,
        }
    }

    pub fn fetch_instruction(&mut self) -> Option<u16> {
        if self.pc < MEMORY_SIZE {
            let config = declare_config();
            let instruction = self.memory[self.pc];
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

    pub fn get_register(&self, index: usize) -> Option<u16> {
        if index < self.registers.len() {
            Some(self.registers[index])
        } else {
            None
        }
    }

    pub fn print_register(&self, index: usize) {
        match self.get_register(index) {
            Some(value) => println!("{}x: {}", integer_to_letter(index), value),
            None => println!("Register index {} is out of bounds.", index),
        }
    }

    pub fn execute_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        let reg1 = ((instruction >> 8) & 0xF) as usize;
        let reg2 = ((instruction >> 4) & 0xF) as usize;
        let value = instruction & 0xFF;

        match opcode {
            ADD_OPCODE => self.registers[reg1] += self.registers[reg2],
            MOV_OPCODE => self.registers[reg1] = value,
            MUL_OPCODE => self.registers[reg1] *= self.registers[reg2],
            SUB_OPCODE => {
                if self.registers[reg1] >= self.registers[reg2] {
                    self.registers[reg1] -= self.registers[reg2];
                } else {
                    neg_num_err("SUB");
                }
            }
            SWAP_OPCODE => self.registers.swap(reg1, reg2),
            DIV_OPCODE => {
                if self.registers[reg2] != 0 {
                    self.registers[reg1] /= self.registers[reg2];
                } else {
                    self.running = false;
                    err_print("Dividing by zero is not allowed.".to_string());
                }
            }
            CLR_OPCODE => self.registers[reg2] = 0,
            INC_OPCODE => self.registers[reg2] += 1,
            DEC_OPCODE => {
                if self.registers[reg2] >= 1 {
                    self.registers[reg2] -= 1;
                } else {
                    neg_num_err("DEC");
                }
            }
            PRINT_OPCODE => self.print_register(reg2),
            POW_OPCODE => self.registers[reg1] = u16::pow(self.registers[reg1], value.into()),
            MOVR_OPCODE => self.registers[reg1] = self.registers[reg2],
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
            file.write_all(&instruction.to_le_bytes())?;
        }
        Ok(())
    }
    pub fn load_binary(&mut self, filename: &str) -> io::Result<()> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;


        for (i, chunk) in buffer.chunks_exact(2).enumerate() {
            if i < MEMORY_SIZE {

                let instruction = u16::from_le_bytes([chunk[0], chunk[1]]);
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