use std::{fs, io, path::Path};

#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    Arithmetic,
    Push,
    Pop,
}

struct Command {
    command_type: CommandType,
    arg1: Option<String>,
    arg2: Option<i32>,
}

struct Parser {
    lines: Vec<String>,
    current: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        let lines: Vec<String> = input
            .lines()
            .map(|line| {
                let line = line.split("//").next().unwrap_or("").trim();
                line.to_string()
            })
            .filter(|line| !line.is_empty())
            .collect();

        Parser { lines, current: 0 }
    }

    fn has_more_commands(&self) -> bool {
        self.current < self.lines.len()
    }

    fn advance(&mut self) {
        if self.has_more_commands() {
            self.current += 1;
        }
    }

    fn command_type(&self) -> Option<CommandType> {
        if !self.has_more_commands() {
            return None;
        }

        let line = &self.lines[self.current];
        let parts: Vec<&str> = line.split_ascii_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => {
                Some(CommandType::Arithmetic)
            }
            "push" => Some(CommandType::Push),
            "pop" => Some(CommandType::Pop),
            _ => None,
        }
    }

    fn arg1(&self) -> Option<String> {
        if !self.has_more_commands() {
            return None;
        }

        let line = &self.lines[self.current];
        let parts: Vec<&str> = line.split_ascii_whitespace().collect();

        match self.command_type()? {
            CommandType::Arithmetic => Some(parts[0].to_string()),
            _ => {
                if parts.len() > 1 {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            }
        }
    }

    fn arg2(&self) -> Option<i32> {
        if !self.has_more_commands() {
            return None;
        }

        let line = &self.lines[self.current];
        let parts: Vec<&str> = line.split_ascii_whitespace().collect();

        match self.command_type()? {
            CommandType::Push | CommandType::Pop => {
                if parts.len() > 2 {
                    parts[2].parse().ok()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn current_command(&self) -> Option<Command> {
        Some(Command {
            command_type: self.command_type()?,
            arg1: self.arg1(),
            arg2: self.arg2(),
        })
    }
}

struct CodeWriter {
    output: Vec<String>,
    filename: String,
    label_counter: i32,
}

impl CodeWriter {
    fn new(filename: &str) -> Self {
        CodeWriter {
            output: Vec::new(),
            filename: filename.to_string(),
            label_counter: 0,
        }
    }

    fn write_arithmetic(&mut self, cmd: &str) {
        match cmd {
            "add" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M".to_string(),
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=D+M".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "sub" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M".to_string(),
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=M-D".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "neg" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=-M".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "eq" | "gt" | "lt" => {
                let jump_condition = match cmd {
                    "eq" => "JEQ",
                    "gt" => "JGT",
                    "lt" => "JLT",
                    _ => unreachable!(),
                };

                let true_label = format!("TRUE_{}", self.label_counter);
                let end_label = format!("END_{}", self.label_counter);
                self.label_counter += 1;

                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M".to_string(),
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M-D".to_string(),
                    format!("@{}", true_label),
                    format!("D;{}", jump_condition),
                    "@SP".to_string(),
                    "A=M".to_string(),
                    "M=0".to_string(),
                    format!("@{}", end_label),
                    "0;JMP".to_string(),
                    format!("({})", true_label),
                    "@SP".to_string(),
                    "A=M".to_string(),
                    "M=-1".to_string(),
                    format!("({})", end_label),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "and" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M".to_string(),
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=D&M".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "or" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "D=M".to_string(),
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=D|M".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            "not" => {
                self.output.extend(vec![
                    "@SP".to_string(),
                    "M=M-1".to_string(),
                    "A=M".to_string(),
                    "M=!M".to_string(),
                    "@SP".to_string(),
                    "M=M+1".to_string(),
                ]);
            }
            _ => {}
        }
    }

    fn write_push_pop(&mut self, command_type: CommandType, segment: &str, index: i32) {
        match command_type {
            CommandType::Push => match segment {
                "argument" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@ARG".to_string(),
                        "A=D+M".to_string(),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "local" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@LCL".to_string(),
                        "A=D+M".to_string(),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "static" => {
                    self.output.extend(vec![
                        format!("@{}.{}", self.filename, index),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "constant" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "this" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@THIS".to_string(),
                        "A=D+M".to_string(),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "that" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@THAT".to_string(),
                        "A=D+M".to_string(),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "pointer" => {
                    let register = if index == 0 { "THIS" } else { "THAT" };

                    self.output.extend(vec![
                        format!("@{}", register),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                "temp" => {
                    self.output.extend(vec![
                        format!("@{}", 5 + index),
                        "D=M".to_string(),
                        "@SP".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M+1".to_string(),
                    ]);
                }
                _ => {}
            },
            CommandType::Pop => match segment {
                "argument" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@ARG".to_string(),
                        "D=D+M".to_string(),
                        "@R13".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        "@R13".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                    ]);
                }
                "local" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@LCL".to_string(),
                        "D=D+M".to_string(),
                        "@R13".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        "@R13".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                    ]);
                }
                "static" => {
                    self.output.extend(vec![
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        format!("@{}.{}", self.filename, index),
                        "M=D".to_string(),
                    ]);
                }
                "this" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@THIS".to_string(),
                        "D=D+M".to_string(),
                        "@R13".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        "@R13".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                    ]);
                }
                "that" => {
                    self.output.extend(vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        "@THAT".to_string(),
                        "D=D+M".to_string(),
                        "@R13".to_string(),
                        "M=D".to_string(),
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        "@R13".to_string(),
                        "A=M".to_string(),
                        "M=D".to_string(),
                    ]);
                }
                "pointer" => {
                    let register = if index == 0 { "THIS" } else { "THAT" };
                    self.output.extend(vec![
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        format!("@{}", register),
                        "M=D".to_string(),
                    ]);
                }
                "temp" => {
                    self.output.extend(vec![
                        "@SP".to_string(),
                        "M=M-1".to_string(),
                        "A=M".to_string(),
                        "D=M".to_string(),
                        format!("@{}", 5 + index),
                        "M=D".to_string(),
                    ]);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn get_output(&self) -> String {
        self.output.join("\n")
    }
}

pub struct VMTranslator;

impl VMTranslator {
    pub fn translate(input: &str, filename: &str) -> String {
        let mut parser = Parser::new(input);
        let mut code_writer = CodeWriter::new(filename);

        while parser.has_more_commands() {
            if let Some(command) = parser.current_command() {
                match command.command_type {
                    CommandType::Arithmetic => {
                        if let Some(ref cmd) = command.arg1 {
                            code_writer.write_arithmetic(cmd);
                        }
                    }
                    CommandType::Push | CommandType::Pop => {
                        if let (Some(segment), Some(index)) = (command.arg1, command.arg2) {
                            code_writer.write_push_pop(command.command_type, &segment, index);
                        }
                    }
                }
            }
            parser.advance();
        }
        code_writer.get_output()
    }

    pub fn translate_file(input_path: &str) -> io::Result<()> {
        let input = fs::read_to_string(input_path)?;
        let filename = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let output_path = Path::new(input_path).with_extension("asm");
        let output = Self::translate(&input, filename);
        fs::write(output_path, output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_add() {
        let input = "push constant 7\npush constant 8\nadd";
        let result = VMTranslator::translate(input, "test");
        assert!(result.contains("D=A"));
        assert!(result.contains("M=D+M"));
    }

    #[test]
    fn test_push_constant() {
        let input = "push constant 17";
        let result = VMTranslator::translate(input, "test");
        assert!(result.contains("@17"));
        assert!(result.contains("D=A"));
    }

    #[test]
    fn test_pop_local() {
        let input = "pop local 0";
        let result = VMTranslator::translate(input, "test");
        assert!(result.contains("@LCL"));
        assert!(result.contains("D+M"));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <input.vm>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];

    VMTranslator::translate_file(input_path).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let output_path = Path::new(input_path).with_extension("asm");
    println!(
        "Translation completed: {} -> {}",
        input_path,
        output_path.display()
    );
}
