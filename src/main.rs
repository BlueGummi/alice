use clap::Parser;
use colorized::*;
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use config::*;

mod config;
mod cpu; 
mod helpers;
mod instructions;
use instructions::*;
use cpu::CPU;
use helpers::*;

#[derive(Parser)]
struct Args {
    /// Output file for the binary
    #[clap(short = 'o', long)]
    output: Option<String>,

    /// Path to the assembly file
    file: String,

    /// Run the binary
    #[clap(short, long)]
    run: bool,
}
// declare config in config.rs
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

fn main() {
    let config = declare_config();
    let mut cpu = CPU::new();

    // Parse command-line arguments
    let args = Args::parse();

    // Check if the -o flag is used for compilation
    if let Some(output_file) = args.output {
        // Read the assembly file
        let program = parse_file(read_file(args.file.clone()));

        if config.verbose_debug {
            println!("{:?}", program);
        }

        // Load the program into the CPU
        cpu.load_program(&program);

        // Emit the binary
        if let Err(e) = cpu.emit_binary(&output_file) {
            eprintln!("Error writing binary file: {}", e);
            return;
        } else {
            println!("Binary emitted to {}", output_file);
        }
        
        return; // Exit after compiling
    }

    // If the -r flag is used, run the specified file
    if args.run {
        let file_to_run = &args.file; // Use the provided file argument

        // Attempt to load the binary file
        if let Err(e) = cpu.load_binary(file_to_run) {
            eprintln!("Error loading binary file: {}", e);

            // If loading the binary fails, assume it's an assembly file and compile it
            let program = parse_file(read_file(file_to_run.clone()));

            if config.verbose_debug {
                println!("{:?}", program);
            }

            // Load the program into the CPU
            cpu.load_program(&program);

            // Emit default output file if not specified
            let output_file = format!("{}.bin", file_to_run);
            if let Err(e) = cpu.emit_binary(&output_file) {
                eprintln!("Error writing binary file: {}", e);
                return;
            } else {
                println!("Binary emitted to {}", output_file);
            }

            // Run the newly created binary
            if let Err(e) = cpu.load_binary(&output_file) {
                eprintln!("Error loading binary file: {}", e);
                return;
            }
            cpu.run();
            return; // Exit after running the binary
        }

        // If it successfully loads the binary, just run it
        cpu.run();
        return; // Exit after running the binary
    }

    // Normal execution flow for assembly if no run flag is used
    let program = parse_file(read_file(args.file.clone()));

    if config.verbose_debug {
        println!("{:?}", program);
    }

    // Load the program into the CPU and run it
    cpu.load_program(&program);
    cpu.run();

    // Print register values if debug is enabled
    if config.debug || config.verbose_debug {
        for (i, &value) in cpu.registers.iter().enumerate() {
            println!(
                "R{}: {}",
                i,
                value.to_string().color(Colors::CyanFg)
            ); // Print out registers
        }
    }

    // Print file contents if debug is enabled
    if config.debug || config.verbose_debug {
        println!(
            "{}\n{}\n",
            "\nFILE CONTENTS".color(Colors::WhiteFg),
            read_file(args.file.clone()).color(Colors::GreenFg)
        );
    }
}
fn read_file(f_name: String) -> String {
    if Path::new(&f_name).exists() {
        fs::read_to_string(&f_name).expect("File found, read unsuccessful.")
    } else {
        println!("Could not find file, file created");
        let default_content = "MOV 1, 5\nMOV 2, 3\nADD 0, 1\nSUB 1, 2\nMUL 1, 2";
        fs::write(&f_name, default_content).expect("Could not write to file");
        default_content.to_string() 
    }
}
// we want to format the assembly like this: INSTRUCTION, DESTINATION, SOURCE

/* TODO: Rewrite file parsing logic to prepare for LOOP instruction
    How to do this?
    Create a second clone of f_contents, add line numbers to each line on this
    When LOOP is read, find the line that the LOOP instruction is looking for, then make a clone
    of the clone and cutoff the clone's clone's contents so that they start at the line
    LOOP is looking for

    Then,
    make the f_contents string identical to the clone's clone, without the leading line numbers.
    if LOOP is called again, refer back to the original clone and rinse and repeat :)
*/
fn parse_file(f_contents: String) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let config = declare_config();
    let c_contents = append_line_numbers(&f_contents);

    if config.verbose_debug {
        println!("File contents with line numbers:\n{}", c_contents);
    }

    for line in f_contents.lines().collect::<Vec<&str>>() {
        // beginning checks, will stop parsing when HALT is found
        if f_contents.is_empty() {
            err_print("Contents of file are empty.".to_string());
        }
        if f_contents[0..4].contains("HALT") {
            if config.verbose_debug {
                println!("HALT detected. Parsing complete.");
            }
            break;
        }

        let line = &remove_comments(&mut line.to_string());
        
        let eol = find_end_of_line(line);
        let (src, dest, instruc, _comma_loc) = extract_components(&mut line.to_string(), eol);
        if config.verbose_debug {
            debug_print(&instruc, &src, &dest, &line.to_string());
        }
        let (dest_i, src_i) = parse_values(src, dest);
        let instruc_slice = &instruc[..];
        // _i means it is type usize
        let instruction = match instruc_slice.to_uppercase().as_str() {
            "ADD" => Instruction::ADD(dest_i, src_i),
            "SUB" => Instruction::SUB(dest_i, src_i),
            "MUL" => Instruction::MUL(dest_i, src_i),
            "MOV" => Instruction::MOV(
                dest_i,
                src_i.try_into().expect("MOV instruction parsing error. Line 351."),
            ),
            "SWAP" => Instruction::SWAP(dest_i, src_i),
            "DIV" => Instruction::DIV(dest_i, src_i),
            "CLR" => Instruction::CLR(dest_i),
            "DEC" => Instruction::DEC(dest_i),
            "INC" => Instruction::INC(dest_i),
            "HALT" => Instruction::HALT,
            "PRINT" => Instruction::PRINT(dest_i),
            "POW" => Instruction::POW(
                dest_i,
                src_i.try_into().expect("POW instruction parsing error. Line 362."),
            ),
            "MOVR" => Instruction::MOVR(dest_i, src_i),
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

// extracts the data for SRC and DEST, returns them 
fn extract_components(f_contents: &mut str, eol: usize) -> (String, String, String, usize) {
    let space_loc = f_contents.find(" ").unwrap(); // space_loc is the location of the space
    let (src, dest, comma_loc) = if f_contents[0..eol].contains(",") {
        let comma_loc = f_contents.find(",").unwrap(); // comma_loc is the location of the comma in assembly
        (
            delete_last_letter(f_contents[space_loc..comma_loc].trim()).to_string(),
            delete_last_letter(f_contents[comma_loc + 1..eol].trim()).to_string(),
            comma_loc,
        )
    } else {
        (
            delete_last_letter(f_contents[space_loc..eol].trim()).to_string(),
            "0".to_string(),
            eol,
        )
    };
    let instruc = f_contents[..space_loc].trim().to_string();
    (src, dest, instruc, comma_loc)
}




fn parse_values(src: String, dest: String) -> (usize, usize) {
    let src_i = if src.contains("b") && has_b_with_num(&src) /* this here to check for BX*/ {
        i32::from_str_radix(&src[2..], 2).expect("Not a binary number!") as usize
    } else if has_single_letter(&src) { // this will handle single letter registers, RA parses to 0, RB to 1, etc.
        let src_char: Vec<char> = src.chars().collect();
        let src = letter_to_integer(*src_char.first().unwrap_or(&' '));
        src.unwrap_or(0) as usize
    } 
    
    else {
        src.parse::<usize>()
            .expect("Failed to convert parsed &str to usize")
    };

    let dest_i = if dest.contains("b") && has_b_with_num(&dest)  /* this here to check for BX*/ {
        i32::from_str_radix(&dest[2..], 2).expect("Not a binary number!") as usize
    } else if has_single_letter(&dest) {
        let dest_char: Vec<char> = dest.chars().collect();
        let dest = letter_to_integer(*dest_char.first().unwrap_or(&' '));
        dest.unwrap_or(0) as usize
    } else {
        dest.parse::<usize>()
            .expect("Failed to convert parsed &str to usize")
    };

    (src_i, dest_i)
}
