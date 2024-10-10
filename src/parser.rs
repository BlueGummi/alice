use crate::*;
use std::fs;
pub fn read_file(f_name: &String) -> String {
    if Path::new(&f_name).exists() {
        match fs::read_to_string(&f_name) {
            Ok(content) => content,
            Err(_) => {
                println!("Error reading file '{}'. Exiting.", f_name);
                std::process::exit(1);
            }
        }
    } else {
        println!("Could not find file; creating it.");
        let default_content = "MOV 1, 5\nMOV 2, 3\nADD 0, 1\nSUB 1, 2\nMUL 1, 2";
        match fs::write(&f_name, default_content) {
            Ok(_) => default_content.to_string(),
            Err(_) => {
                println!("Could not write to file '{}'. Exiting.", f_name);
                std::process::exit(1);
            }
        }
    }
}
// we want to format the assembly like this: INSTRUCTION, DESTINATION, SOURCE

/*  TODO: Rewrite file parsing logic to prepare for LOOP instruction
    How to do this?
    Create a second clone of f_contents, add line numbers to each line on this
    When LOOP is read, find the line that the LOOP instruction is looking for, then make a clone
    of the clone and cutoff the clone's clone's contents so that they start at the line
    LOOP is looking for

    Then,
    make the f_contents string identical to the clone's clone, without the leading line numbers.
    if LOOP is called again, refer back to the original clone and rinse and repeat :)
*/
pub fn parse_file(f_contents: String) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let config = declare_config();
    let c_contents = append_line_numbers(&f_contents);

    if config.verbose_debug {
        println!("File contents with line numbers:\n{}", c_contents);
    }
    let mut line_number = 1;
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
        let (dest_i, src_i) = parse_values(src, dest, line_number);
        let instruc_slice = &instruc[..];
        // _i means it is type usize
        let instruction = match instruc_slice.to_uppercase().as_str() {
            "ADD" => Instruction::ADD(dest_i, src_i),
            "SUB" => Instruction::SUB(dest_i, src_i),
            "MUL" => Instruction::MUL(dest_i, src_i),
            "MOV" => {
                match src_i.try_into() {
                    Ok(value) => Instruction::MOV(dest_i, value),
                    Err(_) => {
                        println!(
                            "ERROR: SOURCE is too large to convert to usize on line {}.",
                            line_number
                        );
                        std::process::exit(0); 
                    }
                }
            }
            "SWAP" => Instruction::SWAP(dest_i, src_i),
            "DIV" => Instruction::DIV(dest_i, src_i),
            "CLR" => Instruction::CLR(dest_i),
            "DEC" => Instruction::DEC(dest_i),
            "INC" => Instruction::INC(dest_i),
            "HALT" => Instruction::HALT,
            "PRINT" => Instruction::PRINT(dest_i),
            "POW" => {
                match src_i.try_into() {
                    Ok(value) => Instruction::POW(dest_i, value),
                    Err(_) => {
                        println!(
                            "ERROR: SOURCE is too large to convert to usize on line {}.",
                            line_number
                        );
                        std::process::exit(0); 
                    }
                }
            }
            "MOVR" => Instruction::MOVR(dest_i, src_i),
            _ => {
                println!(
                    "Assembling ended.\nErr: Unknown instruction: \"{}\" on line {}.",
                    instruc, line_number
                );
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
        line_number += 1;
    }

    instructions.push(Instruction::HALT);

    if config.verbose_debug {
        println!("{:?}", instructions);
    }

    instructions
}

pub fn find_end_of_line(f_contents: &str) -> usize {
    if f_contents.contains("\n") {
        f_contents.find("\n").unwrap() // if newline found
    } else {
        f_contents.len() // if no newline
    }
}

// extracts the data for SRC and DEST, returns them
pub fn extract_components(f_contents: &mut str, eol: usize) -> (String, String, String, usize) {
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

pub fn parse_values(src: String, dest: String, line_number: i32) -> (usize, usize) {
    let src_i = if src.contains("b") && has_b_with_num(&src) {
        match i32::from_str_radix(&src[2..], 2) {
            Ok(value) => value as usize,
            Err(_) => {
                assembler_error("not a valid binary number.", line_number, src);
                std::process::exit(0);
            }
        }
    } else if has_single_letter(&src) {
        // this will handle single letter registers, RA parses to 0, RB to 1, etc.
        let src_char: Vec<char> = src.chars().collect();
        let src = letter_to_integer(*src_char.first().unwrap_or(&' '));
        src.unwrap_or(0) as usize
    } else {
        match dest.parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                assembler_error("not a valid value.", line_number, src);
                std::process::exit(0);
            }
        }
    };

    let dest_i = if dest.contains("b") && has_b_with_num(&dest) {
        // make sure line long enuff
        if dest.len() > 2 {
            match i32::from_str_radix(&dest[2..], 2) {
                Ok(value) => value as usize,
                Err(_) => {
                    assembler_error("not a valid binary number.", line_number, dest);
                    std::process::exit(0);
                }
            }
        } else {
            assembler_error("too short to be a valid binary number.", line_number, dest);
            std::process::exit(0);
        }
    } else if has_single_letter(&dest) {
        let dest_char: Vec<char> = dest.chars().collect();
        let dest = letter_to_integer(*dest_char.first().unwrap_or(&' '));
        dest.unwrap_or(0) as usize
    } else {
        match dest.parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                assembler_error("not a valid value.", line_number, dest);
                std::process::exit(0);
            }
        }
    };

    (src_i, dest_i)
}

