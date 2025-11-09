# Nand2Tetris VM Translator

A Rust implementation of a Virtual Machine translator for the Nand2Tetris course. This tool translates VM (Virtual Machine) code into Hack assembly language.

## Overview

This VM translator is part of the Nand2Tetris project and implements a two-stage compilation process:
- High-level language → VM code (handled by compiler)
- VM code → Assembly code (handled by this translator)

The translator supports stack-based VM commands and translates them into Hack assembly language that can run on the Hack computer platform.

## Features

- **Arithmetic Operations**: `add`, `sub`, `neg`, `eq`, `gt`, `lt`, `and`, `or`, `not`
- **Memory Operations**: `push`, `pop` with support for multiple memory segments
- **Memory Segments**:
  - `constant` - Virtual segment for constants
  - `local` - Function's local variables
  - `argument` - Function arguments
  - `static` - Static variables
  - `this` - Heap object pointer
  - `that` - Array pointer
  - `pointer` - THIS/THAT pointers
  - `temp` - General purpose registers

## Usage

Build the project:
```bash
cargo build --release
```

Translate a VM file to assembly:
```bash
cargo run <input_file.vm>
```

This will generate an output file with the same name but `.asm` extension.

## Example

Input VM code (`test.vm`):
```
push constant 7
push constant 8
add
```

Output assembly code (`test.asm`):
```
@7
D=A
@SP
A=M
M=D
@SP
M=M+1
@8
D=A
@SP
A=M
M=D
@SP
M=M+1
@SP
M=M-1
A=M
D=M
@SP
M=M-1
A=M
M=M+D
@SP
M=M+1
```

## Architecture

The translator consists of two main components:

### Parser
- Parses VM commands from input file
- Filters out comments and empty lines
- Identifies command types and arguments

### CodeWriter
- Generates Hack assembly code for each VM command
- Manages memory segments and stack operations
- Handles label generation for comparison operations

## Testing

Run the test suite:
```bash
cargo test
```

The tests verify correct translation of arithmetic operations, push/pop commands, and memory segment access.

## Implementation Details

- Written in Rust for memory safety and performance
- Follows the VM specification from the Nand2Tetris course
- Generates optimized assembly code for the Hack platform
- Supports all required VM commands for Project 7 of the course

## License

This project is part of the Nand2Tetris educational series.