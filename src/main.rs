use clap::Parser;
use colorized::*;
use config::Config;
use fs::File;
use std::convert::TryInto;
use std::fs;

mod config;
const MEMORY_SIZE: usize = 65536;
// args for CLAP (TODO: IMPLEMENT)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "main.asm")]
    file: String,
}

// Declare config in config.rs
pub fn declare_config() -> Config {
    let config_content = match fs::read_to_string("config.toml") {
        Ok(content) => content,
        Err(_) => {
            return Config::default();
        }
    };

    match toml::de::from_str::<Config>(&config_content) {
        Ok(config) => config,
        Err(_) => {
            println!("config.toml parsing failed. defaulting.");
            Config::default() // return default config if parsing fails
        }
    }
}

// Add instructions here
#[derive(Debug)]
enum Instruction {
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
    HALT,
}

// CPU struct
struct CPU {
    registers: [u16; 8],
    memory: [u16; MEMORY_SIZE],
    pc: usize,
    running: bool,
}

impl CPU {
    fn new() -> CPU {
        CPU {
            registers: [0; 8],
            memory: [0; MEMORY_SIZE],
            pc: 0,          // default program counter to 0
            running: false, // create new CPU when called
        }
    }

    fn load_program(&mut self, program: &[Instruction]) {
        // load the program into the memory of the CPU
        for (i, instruction) in program.iter().enumerate() {
            self.memory[i] = self.encode_instruction(instruction);
        }
    }

    fn encode_instruction(&self, instruction: &Instruction) -> u16 {
        // encode instructions to hex
        match instruction {
            Instruction::ADD(dst, src) => {
                // ADD: 0x1xx
                (0x1 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::MOV(dst, value) => {
                // MOV: 0x2xx
                (0x2 << 12) | ((*dst as u16) << 8) | (*value & 0xFF)
            }
            Instruction::MUL(dst, src) => {
                // MUL: 0x3xx
                (0x3 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::SUB(dst, src) => {
                // SUB: 0x4xx
                (0x4 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::SWAP(dst, src) => {
                // SWAP: 0x5xx
                (0x5 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::DIV(dst, src) => {
                // DIV: 0x6xx
                (0x6 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::CLR(src) => {
                // CLR: 0x7x0
                (0x7 << 12) | ((*src as u16) << 4)
            }
            Instruction::INC(src) => {
                // INC: 0x8
                (0x8 << 12) | ((*src as u16) << 4)
            }
            Instruction::DEC(src) => {
                // DEC: 0x9xx
                (0x9 << 12) | ((*src as u16) << 4)
            }
            Instruction::PRINT(src) => (0xA << 12) | ((*src as u16) << 4), // PRINT: 0xA
            Instruction::POW(dst, value) => {
                // POW: 0xBxx
                (0xB << 12) | ((*dst as u16) << 8) | (*value & 0xFF)
            }
            Instruction::HALT => {
                // HALT: 0x0000
                0x0000
            } /*_ => {
                  // halt
                  0x0000
              }
              */
        }
    }

    fn fetch_instruction(&mut self) -> u16 {
        let config = declare_config();
        let instruction = self.memory[self.pc];
        self.pc += 1; // add to program counter
        if config.verbose_debug {
            println!("PC:\n{:?}", self.pc);
        }
        instruction
    }

    fn get_register(&self, index: usize) -> Option<u16> {
        if index < self.registers.len() {
            Some(self.registers[index])
        } else {
            None // return None if index is out of bounds
        }
    }

    fn print_register(&self, index: usize) {
        match self.get_register(index) {
            Some(value) => println!("R{}: {}", index, value),
            None => println!("Register index {} is out of bounds.", index),
        }
    }
    fn execute_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12; // extract the opcode
        let reg1 = ((instruction >> 8) & 0xF) as usize; // first register
        let reg2 = ((instruction >> 4) & 0xF) as usize; // second register
        let value = (instruction & 0xFF) as u16; // value for MOV instruction
                                                 // match the opcode as hexadecimal values
        match opcode {
            0x1 => {
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
                let temp = self.registers[reg1];
                self.registers[reg1] = self.registers[reg2];
                self.registers[reg2] = temp;
            }
            0x6 => {
                // DIVIDE (DIV)
                if self.registers[reg2] != 0 {
                    // check reg2 to avoid division by zero
                    self.registers[reg1] /= self.registers[reg2];
                } else {
                    self.running = false;
                    eprintln!("Nope, can't divide like that\n");
                    std::process::exit(0);
                }
            }
            0x7 => {
                // CLR
                self.registers[reg1] = 0; // clear register
            }
            0x8 => {
                // INC
                self.registers[reg1] += 1; // add one to register provided
            }
            0x9 => {
                // DEC
                if self.registers[reg1] >= 1 {
                    self.registers[reg1] -= 1;
                } else {
                    neg_num_err("SUB");
                }
            }
            0xA => {
                // PRINT the register
                self.print_register(reg2);
            }
            0xB => {
                // POWER OF, raise first argument to the power of second
                self.registers[reg1] = u16::pow(self.registers[reg1], value.into());
            }
            _ => {
                // halt or Invalid opcode
                self.running = false;
            }
        }
    }

    fn run(&mut self) {
        // run the CPU and use the previous functions
        self.running = true;
        while self.running {
            let instruction = self.fetch_instruction();
            self.execute_instruction(instruction);
        }
    }
}

fn main() {
    let config = declare_config();
    let mut cpu = CPU::new();
    /* yes, you can create programs like this :)
    let program = [
        Instruction::MOV(1, 5), // Move 5 into R1
        Instruction::MOV(2, 3), // Move 3 into R2
        Instruction::ADD(0, 1), // R0 = R0 + R1 (R0 = 5)
        Instruction::SUB(1, 2), // R1 = R1 - R2 (R1 = 2)
        Instruction::MUL(1, 2), // value of R1 is 2, 2*3 = 6, R1 = 6
                                //Instruction::Halt,        // Halt

        parse_file()
    ];
    */
    let args = Args::parse();

    let program = parse_file(read_file(args.file.to_string()));
    if config.verbose_debug {
        println!("{:?}", program);
    }
    cpu.load_program(&program);
    cpu.run();
    if config.debug || config.verbose_debug {
        for i in 0..=cpu.registers.len() - 1 {
            println!(
                "R{}: {}",
                i,
                cpu.registers[i].to_string().color(Colors::CyanFg)
            ); // print out registers
        }
    }
    if config.debug || config.verbose_debug {
        println!(
            "{}\n{}\n",
            "\nFILE CONTENTS".color(Colors::WhiteFg),
            read_file(args.file.to_string()).color(Colors::GreenFg)
        );
    }
}

fn read_file(f_name: String) -> String {
    let contents;
    if path_exists(&f_name) {
        contents =
            fs::read_to_string(format!("{}", f_name)).expect("File found, read unsuccessful.");
        //println!("\n{}\n", "READING FROM FILE".color(Colors::RedFg));
    } else {
        let _ = File::create(&f_name);
        println!("Could not find file, file created");
        fs::write(&f_name, "MOV 1, 5\nMOV 2, 3\nADD 0, 1\nSUB 1, 2\nMUL 1,2") //default ASM code
            .expect("Could not write to file");
        contents =
            fs::read_to_string(format!("{}", f_name)).expect("File found, read unsuccessful.");
    };
    contents
}
// we want to format the assembly like this: INSTRUCTION, SOURCE, DESTINATION

/* TODO: Rewrite file parsing logic to prepare for JMP instruction
    How to do this?
    Create a second clone of f_contents, add line numbers to each line on this
    When JMP is read, find the line that the JMP instruction is looking for, then make a clone
    of the clone and cutoff the clone's clone's contents so that they start at the line
    JMP is looking for

    Then,
    make the f_contents string identical to the clone's clone, without the leading line numbers.
    if JMP is called again, refer back to the original clone and rinse and repeat :)
*/
fn parse_file(mut f_contents: String) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let config = declare_config();
    let c_contents = append_line_numbers(&f_contents);

    if config.verbose_debug {
        println!("File contents with line numbers:\n{}", c_contents);
    }

    for line in f_contents.lines().collect::<Vec<&str>>() {
        if f_contents.is_empty() {
            eprintln!("Error, provided input file is empty.");
            std::process::exit(0);
        }
        if f_contents[0..4].contains("HALT") {
            if config.verbose_debug {
                println!("HALT detected. Parsing complete.");
            }
            break;
        }

        remove_comments(&mut line.to_string());

        let eol = find_end_of_line(&line);
        let (src, dest, instruc, _comma_loc) = extract_components(&mut line.to_string(), eol);

        if config.verbose_debug {
            debug_print(&instruc, &src, &dest, &line.to_string());
        }
        let (src_i, dest_i) = parse_values(src, dest);
        let instruc_slice = &instruc[..];
        let instruction = match instruc_slice {
            "ADD" => Instruction::ADD(dest_i, src_i),
            "SUB" => Instruction::SUB(dest_i, src_i),
            "MUL" => Instruction::MUL(dest_i, src_i),
            "MOV" => Instruction::MOV(
                dest_i,
                src_i.try_into().expect("Something went wrong with MOV"),
            ),
            "SWAP" => Instruction::SWAP(dest_i, src_i),
            "DIV" => Instruction::DIV(dest_i, src_i),
            "CLR" => Instruction::CLR(src_i),
            "DEC" => Instruction::DEC(src_i),
            "INC" => Instruction::INC(src_i),
            "HALT" => Instruction::HALT,
            "PRINT" => Instruction::PRINT(src_i),
            "POW" => Instruction::POW(
                dest_i,
                src_i.try_into().expect("Something went wrong with POW"),
            ),
            _ => {
                println!("Unknown instruction: {}", instruc);
                std::process::exit(0);
            }
        };
        instructions.push(instruction); // push one instruction to the instruction vector to execute

        if !f_contents.trim().contains("\n") {
            if config.debug || config.verbose_debug {
                println!("Finished parsing code.");
            }
            break;
        }
        // f_contents.replace_range(0..eol + 1, ""); // delete line in string
    }

    instructions.push(Instruction::HALT);

    if config.verbose_debug {
        println!("{:?}", instructions);
    }

    instructions
}

fn find_end_of_line(f_contents: &str) -> usize {
    if f_contents.contains("\n") {
        f_contents.find("\n").unwrap() // if newline found
    } else {
        f_contents.len() // if no newline
    }
}

fn extract_components(f_contents: &mut String, eol: usize) -> (String, String, String, usize) {
    let space_loc = f_contents.find(" ").unwrap(); // space_loc is the location of the space
    let (src, dest, comma_loc) = if f_contents[0..eol].contains(",") {
        let comma_loc = f_contents.find(",").unwrap(); // comma_loc is the location of the comma in assembly
        (
            delete_first_letter(f_contents[space_loc..comma_loc].trim()).to_string(),
            delete_first_letter(f_contents[comma_loc + 1..eol].trim()).to_string(),
            comma_loc,
        )
    } else {
        (
            delete_first_letter(f_contents[space_loc..eol].trim()).to_string(),
            "0".to_string(),
            eol,
        )
    };

    let instruc = f_contents[..space_loc].trim().to_string();
    (src, dest, instruc, comma_loc)
}

fn parse_values(src: String, dest: String) -> (usize, usize) {
    let src_i = if src.contains("b") {
        i32::from_str_radix(&src[2..], 2).expect("Not a binary number!") as usize
    } else {
        src.parse::<usize>()
            .expect("Failed to convert parsed &str to usize")
    };

    let dest_i = if dest.contains("b") {
        i32::from_str_radix(&dest[2..], 2).expect("Not a binary number!") as usize
    } else {
        dest.parse::<usize>()
            .expect("Failed to convert parsed &str to usize")
    };

    (src_i, dest_i)
}

fn debug_print(instruc: &String, src: &String, dest: &String, f_contents: &String) {
    println!(
        "\nRemaining f_contents:\n{}",
        f_contents.trim().color(Colors::YellowFg)
    );
    println!("{}", "FOUND INSTRUCTION".color(Colors::BlueFg));
    print!("{}", "INSTRUCTION:".color(Colors::RedFg));
    print!("{}\n", instruc.color(Colors::BrightMagentaFg));
    print!("{}", "SRC:".color(Colors::RedFg));
    print!("{}\n", src.color(Colors::BrightMagentaFg));
    print!("{}", "DEST:".color(Colors::RedFg));
    print!("{}\n\n", dest.color(Colors::BrightMagentaFg));
}

// functions here aren't mission critical, moreso "helper" little functions to get small jobs done :)

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

// this is here for debug, please ignore :)
/*
fn print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}
*/
fn remove_comments(f_contents: &mut String) {
    let mut result = String::new();
    let lines: Vec<&str> = f_contents.lines().collect();

    for line in lines {
        if let Some(comment_loc) = line.find(';') {
            // append the part of the line before the comment
            result.push_str(&line[..comment_loc]);
        } else {
            // if no comment, append the whole line
            result.push_str(line);
        }
        result.push('\n'); // add a newline back to the result
    }

    // update the original string with the new string without comments
    *f_contents = result;
}

fn delete_first_letter(s: &str) -> &str {
    if !s.is_empty() {
        // check if the first character is a letter
        let first_char = s.chars().next().unwrap();
        if first_char.is_alphabetic() {
            return &s[1..]; // return a slice starting from the second character
        }
    }
    s // return the original string if it's empty or the first character is not a letter
}

fn append_line_numbers(input: &str) -> String {
    input
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{} {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn neg_num_err(instruction: &str) {
    eprintln!(
        "{}{}{}",
        "ERROR, ".color(Colors::RedFg),
        instruction.color(Colors::YellowFg),
        " WILL RESULT IN NEGATIVE NUMBER.\nTERMINATING.".color(Colors::RedFg)
    );
    std::process::exit(0);
}
