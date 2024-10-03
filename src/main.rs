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
    ADD(usize, usize), // (destination, source)
    MOV(usize, u16),   // (destination, immediate value)
    MUL(usize, usize), // (destination, source)
    SUB(usize, usize), // (destination, source)
    SWAP(usize, usize), // (reg1, reg2)
    DIV(usize, usize),
    CLR(usize), // clear one register
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
            pc: 0, // default program counter to 0
            running: false, // create new CPU when called
        }
    }

    fn load_program(&mut self, program: &[Instruction]) { // load the program into the memory of the CPU
        for (i, instruction) in program.iter().enumerate() {
            self.memory[i] = self.encode_instruction(instruction);
        }
    }

    fn encode_instruction(&self, instruction: &Instruction) -> u16 { // encode instructions to binary
        match instruction {
            Instruction::ADD(dst, src) => {
                (0b001 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::MOV(dst, value) => (0b010 << 12) | ((*dst as u16) << 8) | (*value & 0xFF),
            Instruction::MUL(dst, src) => {
                (0b011 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::SUB(dst, src) => {
                (0b100 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::SWAP(dst, src) => {
                (0b101 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::DIV(dst, src) => {
                (0b110 << 12) | ((*dst as u16) << 8) | ((*src as u16) << 4)
            }
            Instruction::CLR(src) => {
                (0b111 << 12) | ((*src as u16) << 4)
            }
            Instruction::HALT => 0x0000, // for MOV, or any values where raw data is taken, do & 0xFF (i think)
        }
    }

    fn fetch_instruction(&mut self) -> u16 {
        let config = declare_config();
        let instruction = self.memory[self.pc];
        self.pc += 1;// add to program counter
        if config.verbose_debug {
            println!("{:?}", self.pc);
        }
        instruction 
    }

    fn execute_instruction(&mut self, instruction: u16) {
        let opcode = instruction >> 12;
        let reg1 = ((instruction >> 8) & 0xF) as usize;
        let reg2 = ((instruction >> 4) & 0xF) as usize;
        let value = (instruction & 0xFF) as u16;
        // add what the opcode does, reg1 is the second argument, reg2 is the first
        match opcode {
            0b001 => {
                // ADD
                self.registers[reg1] += self.registers[reg2];
            }
            0b010 => {
                // MOV
                self.registers[reg1] = value;
            }
            0b011 => {
                // MUL
                self.registers[reg1] *= self.registers[reg2];
            }
            0b100 => {
                // SUB
                self.registers[reg1] -= self.registers[reg2];
            }
            0b101 => {
                // SWAP
                let temp = self.registers[reg1];
                self.registers[reg1] = self.registers[reg2];
                self.registers[reg2] = temp;
            }
            0b110 => {
                // DIVIDE (DIV)
                if self.registers[reg1] != 0 {
                    self.registers[reg2] /= self.registers[reg1];
                } else {
                    self.running = false;
                    eprintln!("Nope, can't divide like that\n");
                    std::process::exit(0);
                }
            }
            0b111 => {
                self.registers[reg1] = 0; //clear
            }
            _ => {
                // Halt or Invalid opcode
                self.running = false;
            }
        }
    }

    fn run(&mut self) { // run the CPU and use the previous functions
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
    println!(
        "\nR0: {}",
        cpu.registers[0].to_string().color(Colors::CyanFg)
    );
    println!("R1: {}", cpu.registers[1].to_string().color(Colors::CyanFg));
    println!("R2: {}", cpu.registers[2].to_string().color(Colors::CyanFg));
    println!("R3: {}", cpu.registers[3].to_string().color(Colors::CyanFg));
    println!("R4: {}", cpu.registers[4].to_string().color(Colors::CyanFg));
    println!("R5: {}", cpu.registers[5].to_string().color(Colors::CyanFg));
    println!("R6: {}", cpu.registers[6].to_string().color(Colors::CyanFg));
    println!("R7: {}", cpu.registers[7].to_string().color(Colors::CyanFg)); //this code is written like this as i can see where i am in the program with ugly code
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
    let mut comma_loc;
    let mut space_loc;
    let mut src;
    let mut eol;
    let mut dest;
    let mut instruc;
    let mut c_contents = append_line_numbers(&f_contents);
    if config.verbose_debug {
        println!("File contents with line numbers:\n{}", c_contents);
    }
    loop {
        if f_contents.is_empty(){
            eprintln!("Error, provided input file is empty."); 
            std::process::exit(0);
        }
        if f_contents[0..4].contains("HALT") {
            if config.verbose_debug {
                println!("HALT detected. Parsing complete.");
            }
            break;
        }
        remove_comments(&mut f_contents);

        comma_loc = f_contents.find(",").unwrap(); // comma_loc is the location of the comma in assembly
        space_loc = f_contents.find(" ").unwrap(); // space_loc is the location of the space
        if f_contents.contains("\n") {
            eol = f_contents.find("\n").unwrap(); // if newline found
        } else {
            eol = f_contents.len(); // if no newline
        }
        src = delete_first_letter(f_contents[space_loc..comma_loc].trim());  // src will find the first value
        dest = delete_first_letter(f_contents[comma_loc + 1..eol].trim());
        instruc = f_contents[..space_loc].trim();
        if config.verbose_debug {
            /// colorful stuff to print, debug
            print!("{}\n", "FOUND INSTRUCTION".color(Colors::BlueFg));
            print!("{}", "INSTRUCTION:".color(Colors::RedFg));
            print!("{}\n", instruc.color(Colors::BrightMagentaFg));
            print!("{}", "SRC:".color(Colors::RedFg));
            print!("{}\n", src.color(Colors::BrightMagentaFg));
            print!("{}", "DEST:".color(Colors::RedFg));
            print!("{}\n", dest.color(Colors::BrightMagentaFg));
            println!("Remaining f_contents:\n{}\n", f_contents.color(Colors::YellowFg));
        }
        // variables suffixed with _i are of type usize
        let dest_i = dest
            .parse::<usize>()
            .expect("Failed to convert parsed &str to usize");
        let src_i = src
            .parse::<usize>()
            .expect("Failed to convert parsed &str to usize");
        // attempt to convert usize to u16
        let src_u16: Result<u16, _> = src_i.try_into();

        // check the result and assign to a variable
        match src_u16 {
            Ok(v) => {}
            Err(_) => {
                println!("Value is too large to fit in a u16!");
            }
        }
        let instruction = match instruc {
            "ADD" => Instruction::ADD(dest_i, src_i),
            "SUB" => Instruction::SUB(dest_i, src_i),
            "MUL" => Instruction::MUL(dest_i, src_i),
            "MOV" => Instruction::MOV(dest_i, src_u16.expect("Something went wrong")),
            "SWAP" => Instruction::SWAP(dest_i, src_i),
            "DIV" => Instruction::DIV(dest_i, src_i),
            "HALT" => Instruction::HALT,
            // add more instruction matches as needed
            _ => {
                println!("Unknown instruction: {}", instruc);
                std::process::exit(0); // Skip unknown instructions
            }
        };
        instructions.push(instruction);
        if !f_contents.trim().contains("\n") {
            println!("Finished parsing code.");
            break; // push one instruction to the instruction vector to execute
        }

        f_contents.replace_range(0..eol + 1, ""); // delete line in string
    }
    instructions.push(Instruction::HALT);
    if config.verbose_debug {
        println!("{:?}", instructions);
    }
    instructions
}



// functions here aren't mission critical, moreso "helper" little functions to get small jobs done :)

fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

// this is here for debug, please ignore :)
fn print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}

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
