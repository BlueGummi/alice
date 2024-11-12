# THIS IS AN OLD, DEPRECATED, POC PROJECT.





# Overview

ALICE, the Architecture Language Interpreter and CPU Emulator.

This program is a **16 bit CPU emulator**, capable of **assembling assembly** files written for it and **running the binaries**, or **assembling and running** the assembly without emitting a binary.

# Installation
This program currently **does not have binaries**, thus, it **must be built from source**.

## Installation prerequisites
To build this program, RustC (the Rust Compiler), git, and vargo must be installed on the system.
Installation commands (OS-agnostic):

`git clone https://github.com/BlueGummi/alice.git`

`cd alice`

`cargo build --release`

To simplify running the binary, an **alias** can be created to the binary located at `./target/release/alice` (or alice.exe if the host system is Windows).

`alias cpu='./target/release/alice'`

Or the equivalent for **Windows**,
`set-alias cpu` and enter `target/release/alice.exe`

# How does it work?

This program is capable of assembling assembly code, running binaries, or directly running assembly without the production of a binary.

**To compile an existing assembly program** written for this CPU emulator, run
`cpu -o <BINARY> <SOURCE>`
e.g. `cpu -o main main.asm`

**To run a pre-existing binary** assembled by this assembler, run
`cpu -r <BINARY>`
e.g. `cpu -r main`

**To directly run a assembly program** without producing a binary, simply run
`cpu <SOURCE>` without passing any flags.
e.g. `cpu main.asm`

This CPU is **Little-Endian**, similar to most real-life CPUs.
The instructions are formatted into binary like this:

Instruction gets bitshifted by **12** bits to the left.
The DESTination gets bitshifted by **8** bits to the left.
The SOURCE gets bitshifted by **4** bits to the left.

When binaries are executed, the emulated CPU will **load the entire binary into the emulated memory**.

The CPU contains a program counter (PC), which **increments by one** for each instruction exectued.

Thus, when the CPU is run, it will check each line of memory, and for each instruction it finds, it will execute it and increment the PC by one, so the next CPU cycle will run the following instruction in the memory.

However, the CPU will **automatically halt** if a certain condition is detected, such as attempting to perform a subtraction operation if a **negative result is detected**.

The assembler will also **automatically append HALT** to the end of each assembly program, thus it is not necessary to write HALT at the end of a program.

# Writing the assembly

## Syntax:

Assembly written for this CPU must be written in this format for instructions with two fields:

`INSTRUCTION, DESTINATION, SOURCE`

e.g. `mov cx, 5`

This CPU has **16** registers, which are **unsigned 16-bit integers**, which can be referenced by **letters** in the assembly code.
e.g., register 0 maps to ax, register 1 maps to bx, etc.

The letter following the register can be anything, so **it is possible to write**
`mov ci, 5` as the trailing letter of the register will be cut off and the register's first letter will be converted to an integer, corresponding to a CPU register.

This CPU is a simple **16 bit machine**, and the instruction opcodes are formatted in **hexadecimal**.

Comments are also supported, and must be prefixed with `;`

# Instructions:

The instructions can be found in src/instructions.rs, and I will add comments to it (if I remember to :skull:), so if this file is outdated, instructions.rs can be viewed to see which instructions the CPU can execute.

As of writing, this CPU supports the **following instructions**, which take the following arguments to produce the following result.

This example line of assembly will be used for each instruction to demonstrate how it is used.

`INSTRUCTION, bx, ax`

or, for instructions that **do not take a register as the second parameter**,

`INSTRUCTION, bx, NUMBER`

or, for instructions that take **one** input parameter,

`INSTRUCTION, ax`

## ADD - OPCODE: 0x1
**Adds** the value of ax to bx, and stores the result in bx.

`add dx, cx`

## MOV - OPCODE: 0x2
**Moves** the value of the NUMBER to bx

`mov bx, 2`

## MUL - OPCODE 0x3
**Multiplies** the values of ax and bx, and stores the result in bx.

`mul bx, ax`

## SUB - OPCODE: 0x4
**Subtracts** the value of ax from bx, and stores the result in bx.

***Warning***: If a SUB operation with a negative result is attempted, the assembler will assemble the code, however the CPU will produce an error when the binary is ran.

`sub dx, ax`

## SWAP - OPCODE: 0x5
**Swaps** the values of ax and bx.

`swap ax, bx`

## DIV - OPCODE: 0x6
Performs **unsigned integer division**, divides ax by bx, stores the result in bx.

`div bx, cx`

## CLR - OPCODE: 0x7
**Clears** the register by resetting the value to 0.

`clr ax`

## INC - OPCODE: 0x8
**Increments** the register's value by 1.

`inc ax`

## DEC - OPCODE: 0x9
**Decrements** the register's value by 1. 

***Warning***: If a DEC operation with a negative result is attempted, the assembler will assemble the code, however the CPU will produce an error when the binary is ran.

`dec ax`

## PRINT - OPCODE: 0xa
**Prints** the value of a register.

`print ax`

## POW - OPCODE: 0xb
**Raises** the value of a register to the power of the second argument provided.

`pow ax, 2`

## MOVR - OPCODE: 0xc
**Similar to MOV**, however this takes a **register as the value to be moved**.

`movr bx, ax`

## CMP - OPCODE: 0xd
**Compares** the values of two registers. If they are the same, the ZFlag on the CPU is set to `true` or `1`, if not, nothing happens.

`cmp ax, bx`

## HALT - OPCODE: 0x0
**Stops** the CPU. Assembler will also stop assembling instructions detected after HALT (yes this is a bug, yes I need to fix it)



