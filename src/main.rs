use clap::Parser;
use colorized::*;
use std::{
    fs,
    path::Path,
    convert::TryInto,
};
mod config;
mod cpu; 
mod helpers;
mod instructions;
mod parser;
use parser::*;
use instructions::*;
use cpu::*;
use helpers::*;
use config::*;

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
        let program = parse_file(read_file(&args.file));

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
            let program = parse_file(read_file(&file_to_run.to_string()));

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
    let program = parse_file(read_file(&args.file));

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
            read_file(&args.file).color(Colors::GreenFg)
        );
    }
}
