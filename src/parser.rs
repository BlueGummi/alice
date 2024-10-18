use crate::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Reads the contents of a file or creates it with default content.
pub fn read_file(f_name: &String) -> String {
    // Check if the file exists at the given path
    if Path::new(&f_name).exists() {
        // If it exists, read the contents of the file
        fs::read_to_string(f_name).unwrap_or_else(|_| {
            // If reading fails, print an error and exit
            println!("Error reading file '{}'. Exiting.", f_name);
            std::process::exit(1);
        })
    } else {
        // If the file does not exist, create it with default content
        println!("Could not find file; creating it.");
        let default_content = "MOV 1, 5\nMOV 2, 3\nADD 0, 1\nSUB 1, 2\nMUL 1, 2";
        fs::write(f_name, default_content).unwrap_or_else(|_| {
            // If writing fails, print an error and exit
            println!("Could not write to file '{}'. Exiting.", f_name);
            std::process::exit(1);
        });
        // Return the default content as a string
        default_content.to_string()
    }
}

/// Lexer to tokenize the assembly code.
fn lex(input: &str) -> Vec<Vec<String>> {
    input
        .lines() // Split input into lines
        .map(|line| {
            // Clean the line by removing comments and splitting it into tokens
            let clean_line = line.split(';').next().unwrap_or(line); // Ignore comments
            clean_line
                .split_whitespace() // Split line into words/tokens based on whitespace
                .filter(|token| !token.is_empty()) // Ignore empty tokens
                .map(|token| token.to_string()) // Convert tokens from &str to String
                .collect() // Collect tokens into a Vec<String>
        })
        .collect() // Collect all lines of tokens into a Vec<Vec<String>>
}

/// Parses the tokenized lines into instructions, handling functions internally.
pub fn parse_file(f_contents: String) -> Vec<Instruction> {
    let mut instructions = Vec::new(); // Vector to store parsed instructions
    let mut functions = HashMap::new(); // Map to store functions and their instructions
    let config = declare_config(); // Obtain configuration settings
    let tokens = lex(&f_contents); // Tokenize the input contents
    let mut current_function: Option<String> = None; // Track the current function being defined
    let mut current_function_instructions = Vec::new(); // Store instructions for the current function

    if config.verbose_debug {
        // If verbose debugging is enabled, print the tokenized instructions
        println!("Tokenized instructions:\n{:?}", tokens);
    }

    // Iterate over the tokenized lines
    for (line_number, tokens) in tokens.iter().enumerate() {
        if tokens.is_empty() {
            continue; // Skip empty lines
        }

        // Check if the first token indicates the start of a function
        if tokens[0].starts_with('.') {
            if tokens[0] == ".end" {
                // Handle the end of a function
                if let Some(func_name) = current_function.take() {
                    // Insert the function's instructions into the map
                    functions.insert(func_name, current_function_instructions);
                    current_function_instructions = Vec::new(); // Reset for the next function
                } else {
                    // Error if .end is found without a corresponding function
                    println!(
                        "Error: .end without a corresponding function on line {}.",
                        line_number
                    );
                    std::process::exit(0);
                }
            } else {
                // Start a new function
                if current_function.is_none() {
                    current_function = Some(tokens[0].to_string()); // Store the function name
                } else {
                    // Error if nested function definitions are found
                    println!(
                        "Error: Nested function definitions are not allowed on line {}.",
                        line_number
                    );
                    std::process::exit(0);
                }
            }
        } else if let Some(ref _func_name) = current_function {
            // Collect instructions for the current function
            if let Some(instruction) = parse_instruction(tokens, line_number as i32) {
                current_function_instructions.push(instruction); // Add instruction to the current function
            }
        } else if let Some(instruction) = parse_instruction(tokens, line_number as i32) {
            // Add instruction to the global instructions
            instructions.push(instruction);
        }
    }

    if config.verbose_debug {
        // Print global instructions and functions if verbose debugging is enabled
        println!("Global instructions: {:?}", instructions);
        println!("Functions: {:?}", functions);
    }

    // Ensure HALT instruction is at the end of global instructions
    //instructions.push(Instruction::HALT);

    instructions // Return the collected instructions
}

/// Parses a single instruction from tokens.
fn parse_instruction(tokens: &[String], line_number: i32) -> Option<Instruction> {
    if tokens.is_empty() {
        return None; // Return None if no instruction is found
    }
    let instruc = &tokens[0]; // Get the instruction name
    let (dest, src): (u16, u16) = parse_operands(tokens); // Parse destination and source operands

    // Match the instruction name and create the appropriate Instruction variant
    match instruc.to_uppercase().as_str() {
        "ADD" => Some(Instruction::ADD(dest, src)),
        "SUB" => Some(Instruction::SUB(dest, src)),
        "MUL" => Some(Instruction::MUL(dest, src)),
        "MOV" => {
            // Function to create a MOV instruction based on the destination and source
            fn create_mov_instruction(dest: u16, src: Option<&str>) -> Instruction {
                match src {
                    Some(value) => {
                        // Try to parse the source value as u16
                        if let Ok(parsed_value) = value.parse::<u16>() {
                            Instruction::MOV(dest, parsed_value) // Move immediate value
                        } else {
                            // If parsing fails, treat the value as a register
                            let reg_index =
                                letter_to_integer(value.chars().next().unwrap_or(' ')).unwrap_or(0);
                            Instruction::MOVR(dest, reg_index.into()) // Move from register
                        }
                    }
                    None => Instruction::MOV(dest, 0), // Default to moving 0 if src is None
                }
            }

            // Convert dest and src to u16 and call create_mov_instruction
            let instruction = create_mov_instruction(dest, Some(&tokens[2]));
            Some(instruction)
        }
        "SWAP" => Some(Instruction::SWAP(dest, src)),
        "DIV" => Some(Instruction::DIV(dest, src)),
        "CLR" => Some(Instruction::CLR(dest)),
        "DEC" => Some(Instruction::DEC(dest)),
        "INC" => Some(Instruction::INC(dest)),
        "CMP" => Some(Instruction::CMP(dest, src)),
        "HALT" => Some(Instruction::HALT),
        "PRINT" => Some(Instruction::PRINT(dest)),
        "POW" => Some(Instruction::POW(dest, src)),
        "MOVR" => Some(Instruction::MOVR(dest, src)),
        "JMP" => Some(Instruction::JMP(dest)),
        "NOP" => Some(Instruction::NOP),
        _ => {
            // Handle unknown instructions
            println!(
                "Error: Unknown instruction: \"{}\" on line {}.",
                instruc, line_number
            );
            std::process::exit(0); // Exit if an unknown instruction is encountered
        }
    }
}

/// Parses the operands from the tokenized line.
fn parse_operands(tokens: &[String]) -> (u16, u16) {
    // Parse the first and second operands from the tokens, defaulting to 0 if not found
    let dest = parse_value(tokens.get(1).unwrap_or(&"0".to_string()));
    let src = parse_value(tokens.get(2).unwrap_or(&"0".to_string()));
    (dest, src) // Return the parsed operands as a tuple
}

/// Converts a token into a u16 value, handling both numeric and register inputs.
fn parse_value(token: &String) -> u16 {
    // Check if the token is a binary number
    if token.starts_with("b") && has_b_with_num(token) {
        i32::from_str_radix(&token[2..], 2).unwrap_or_else(|_| {
            // Handle invalid binary numbers
            println!("Error: Not a valid binary number: {}", token);
            std::process::exit(0);
        }) as u16 // Cast to u16
    } else if let Ok(value) = token.parse::<u16>() {
        // Attempt to parse the token as a u16
        value
    } else {
        // If parsing fails, treat the token as a register
        letter_to_integer(token.chars().next().unwrap_or(' '))
            .unwrap_or(0)
            .into() // Convert to u16
    }
}
