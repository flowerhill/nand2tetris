# Nand2Tetris - The Elements of Computing Systems

Implementation of "The Elements of Computing Systems" course projects in Rust.

## Projects

- **nand2tetris-asm/**: Hack assembly language assembler
  - Converts assembly language to machine language
- **nand2tetris-vm/**: Jack virtual machine translator
  - Translates high-level language to assembly language

## Usage

Each project can be built and run independently:

```bash
cd nand2tetris-asm
cargo build --release
cargo run -- input.asm
```

```bash
cd nand2tetris-vm
cargo build --release
cargo run -- input.vm
```
