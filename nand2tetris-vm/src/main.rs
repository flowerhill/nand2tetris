use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use std::{fs, path::Path};

fn validate_label(label: &str) -> Result<()> {
    ensure!(!label.is_empty(), "label name cannot be empty");

    let re = Regex::new(r"^[a-zA-Z_.:][a-zA-Z0-9_.:]*$").unwrap();

    ensure!(
        re.is_match(label),
        "Invalid label name '{}': must start with letter or underscore, \
            and contain only letters, digits, '_', '.', ':'",
        label
    );

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    Arithmetic,
    Push,
    Pop,
    Label,
    Goto,
    IfGoto,
    Call,
    Function,
    Return,
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

    fn parse(&self) -> Result<Command> {
        ensure!(self.has_more_commands(), "No more commands availavle");

        let line = &self.lines[self.current];
        let parts: Vec<&str> = line.split_ascii_whitespace().collect();

        let cmd_name = parts.get(0).context("Empty command")?;

        match *cmd_name {
            "add" | "sub" | "neg" | "eq" | "gt" | "lt" | "and" | "or" | "not" => Ok(Command {
                command_type: CommandType::Arithmetic,
                arg1: Some(cmd_name.to_string()),
                arg2: None,
            }),
            "push" => {
                let segment = parts
                    .get(1)
                    .context("Missing segment argument for 'push' command")?;
                let index = parts
                    .get(2)
                    .context("Missing segment argument for 'push' command")?
                    .parse()
                    .context(format!(
                        "Invalid index: '{}' is not a valid integer",
                        parts[2]
                    ))?;
                Ok(Command {
                    command_type: CommandType::Push,
                    arg1: Some(segment.to_string()),
                    arg2: Some(index),
                })
            }
            "pop" => {
                let segment = parts
                    .get(1)
                    .context("Missing segment argument for 'push' command")?;
                let index = parts
                    .get(2)
                    .context("Missing segment argument for 'push' command")?
                    .parse()
                    .context(format!(
                        "Invalid index: '{}' is not a valid integer",
                        parts[2]
                    ))?;
                Ok(Command {
                    command_type: CommandType::Pop,
                    arg1: Some(segment.to_string()),
                    arg2: Some(index),
                })
            }
            "label" => {
                let label = parts
                    .get(1)
                    .context("Missing label name for 'label' command")?;
                validate_label(label).context(format!("Invalid label in 'label' command"))?;

                Ok(Command {
                    command_type: CommandType::Label,
                    arg1: Some(label.to_string()),
                    arg2: None,
                })
            }
            "goto" => {
                let label = parts
                    .get(1)
                    .context("Missing label name for 'goto' command")?;
                validate_label(label).context(format!("Invalid label in 'goto' command"))?;

                Ok(Command {
                    command_type: CommandType::Goto,
                    arg1: Some(label.to_string()),
                    arg2: None,
                })
            }
            "if-goto" => {
                let label = parts
                    .get(1)
                    .context("Missing label name for 'if-goto' command")?;
                validate_label(label).context(format!("Invalid label in 'if-goto' command"))?;

                Ok(Command {
                    command_type: CommandType::IfGoto,
                    arg1: Some(label.to_string()),
                    arg2: None,
                })
            }
            "call" => {
                let f_name = parts
                    .get(1)
                    .context("Missing function for 'call' command")?;
                let n_vars: i32 = parts
                    .get(2)
                    .context("Missing local variable count for 'call' command")?
                    .parse()
                    .context("Invalid number for variable count")?;

                Ok(Command {
                    command_type: CommandType::Call,
                    arg1: Some(f_name.to_string()),
                    arg2: Some(n_vars),
                })
            }
            "function" => {
                let f_name = parts
                    .get(1)
                    .context("Missing function for 'call' command")?;
                let n_vars: i32 = parts
                    .get(2)
                    .context("Missing local variable count for 'call' command")?
                    .parse()
                    .context("Invalid number for variable count")?;

                Ok(Command {
                    command_type: CommandType::Function,
                    arg1: Some(f_name.to_string()),
                    arg2: Some(n_vars),
                })
            }
            _ => bail!(format!("Unkonown command: '{}'", cmd_name)),
        }
    }

    fn current_line_number(&self) -> usize {
        self.current + 1
    }
}

struct CodeWriter {
    output: Vec<String>,
    filename: String,
    label_counter: i32,
    call_counter: i32,
}

impl CodeWriter {
    fn new(filename: &str) -> Self {
        CodeWriter {
            output: Vec::new(),
            filename: filename.to_string(),
            label_counter: 0,
            call_counter: 0,
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
            _ => unreachable!(),
        }
    }

    fn write_push(&mut self, segment: &str, index: i32) {
        match segment {
            "argument" => {
                self.push_segment("ARG", index);
            }
            "local" => {
                self.push_segment("LCL", index);
            }
            "static" => {
                self.push_value(&format!("{}.{}", self.filename, index), false);
            }
            "constant" => {
                self.push_value(&index.to_string(), true);
            }
            "this" => {
                self.push_segment("THIS", index);
            }
            "that" => {
                self.push_segment("THAT", index);
            }
            "pointer" => {
                let register = if index == 0 { "THIS" } else { "THAT" };
                self.push_value(register, false);
            }
            "temp" => {
                self.push_value(&(5 + index).to_string(), false);
            }
            _ => unreachable!(),
        }
    }

    fn write_pop(&mut self, segment: &str, index: i32) {
        match segment {
            "argument" => {
                self.pop_segment("ARG", index);
            }
            "local" => {
                self.pop_segment("LCL", index);
            }
            "static" => {
                self.pop_direct(&format!("{},{}", self.filename, index));
            }
            "this" => {
                self.pop_segment("THIS", index);
            }
            "that" => {
                self.pop_segment("THAT", index);
            }
            "pointer" => {
                let register = if index == 0 { "THIS" } else { "THAT" };
                self.pop_direct(register);
            }
            "temp" => {
                self.pop_direct(&(5 + index).to_string());
            }
            _ => unreachable!(),
        }
    }

    fn write_label(&mut self, label: &str) {
        self.output.push(format!("({})", label));
    }

    fn write_goto(&mut self, label: &str) {
        self.output.push(format!("@{}", label));
        self.output.push("0;JMP".to_string());
    }

    fn write_if_goto(&mut self, label: &str) {
        self.output.extend(vec![
            "@SP".to_string(),
            "M=M-1".to_string(),
            "A=M".to_string(),
            "D=M".to_string(),
            format!("@{}", label),
            "D;JNE".to_string(),
        ]);
    }

    fn write_call(&mut self, function_name: &str, n_args: i32) {
        let return_address_symbol = format!("{}$ret{}", function_name, self.call_counter);
        self.push_value(&return_address_symbol, true);

        for register in ["LCL", "ARG", "THIS", "THAT"] {
            self.push_value(register, false);
        }

        // ARGを引数の最初の座標を指すようにする
        // returnAddress, LCL, ARG, THIS, THAT と nArgs分SPをインクリメントしているので、
        // SP - 5 - nArgsでArgの最初の座標を指す
        self.output.extend(vec![
            "@SP".to_string(),
            "D=M".to_string(),
            format!("@{}", 5 + n_args),
            "D=D-A".to_string(),
            "@ARG".to_string(),
            "M=D".to_string(),
        ]);

        self.output.extend(vec![
            "@SP".to_string(),
            "D=M".to_string(),
            "@LCL".to_string(),
            "M=D".to_string(),
        ]);

        self.write_goto(function_name);

        self.output.push(format!("{return_address_symbol}"));
    }

    fn write_function(&mut self, label: &str) {
        todo!()
    }

    fn write_return(&mut self, label: &str) {
        todo!()
    }

    fn get_output(&self) -> String {
        self.output.join("\n")
    }

    // 値を直接push（定数またはレジスタの値）
    fn push_value(&mut self, value: &str, is_address: bool) {
        let address = if is_address { "A" } else { "M" };
        self.output.extend(vec![
            format!("@{value}"),
            format!("D={address}"),
            "@SP".to_string(),
            "A=M".to_string(),
            "M=D".to_string(),
            "@SP".to_string(),
            "M=M+1".to_string(),
        ]);
    }

    // ベースアドレス + index の値をpush
    fn push_segment(&mut self, base: &str, index: i32) {
        self.output.extend(vec![
            format!("@{}", index),
            "D=A".to_string(),
            format!("@{}", base),
            "A=D+M".to_string(),
            "D=M".to_string(),
            "@SP".to_string(),
            "A=M".to_string(),
            "M=D".to_string(),
            "@SP".to_string(),
            "M=M+1".to_string(),
        ]);
    }

    // スタックからpopして直接アドレスに格納
    fn pop_direct(&mut self, address: &str) {
        self.output.extend(vec![
            "@SP".to_string(),
            "M=M-1".to_string(),
            "A=M".to_string(),
            "D=M".to_string(),
            format!("@{}", address),
            "M=D".to_string(),
        ]);
    }

    // スタックからpopしてベースアドレス + index に格納
    fn pop_segment(&mut self, base: &str, index: i32) {
        self.output.extend(vec![
            format!("@{}", index),
            "D=A".to_string(),
            format!("@{}", base),
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
}

pub struct VMTranslator;

impl VMTranslator {
    pub fn translate(input_path: &str, output_path: &str) -> Result<String> {
        let mut parser = Parser::new(input_path);
        let mut code_writer = CodeWriter::new(output_path);

        while parser.has_more_commands() {
            let line_num = parser.current_line_number();

            let cmd = parser.parse().context(format!("Line {}", line_num))?;

            match cmd.command_type {
                CommandType::Arithmetic => {
                    let op = cmd.arg1.context("Missing arithmetic operatioin")?;

                    code_writer.write_arithmetic(&op);
                }
                CommandType::Push => {
                    let segment = cmd.arg1.context("Missing segment")?;
                    let index = cmd.arg2.context("Missing segment")?;
                    code_writer.write_push(&segment, index);
                }
                CommandType::Pop => {
                    let segment = cmd.arg1.context("Missing segment")?;
                    let index = cmd.arg2.context("Missing segment")?;
                    code_writer.write_pop(&segment, index);
                }
                CommandType::Label => {
                    let label = cmd.arg1.context("Missing label")?;
                    code_writer.write_label(&label);
                }
                CommandType::Goto => {
                    let label = cmd.arg1.context("Missing goto label")?;
                    code_writer.write_goto(&label);
                }
                CommandType::IfGoto => {
                    let label = cmd.arg1.context("Missing if-goto label")?;
                    code_writer.write_if_goto(&label);
                }
                CommandType::Call => todo!(),
                CommandType::Function => todo!(),
                CommandType::Return => todo!(),
            }
            parser.advance();
        }

        Ok(code_writer.get_output())
    }

    fn translate_file(input_path: &str) -> Result<()> {
        let input = fs::read_to_string(input_path)
            .context(format!("Failed to read file '{}'", input_path))?;
        let filename = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid pattern")?;

        let output = Self::translate(&input, filename)?;
        let output_path = Path::new(input_path).with_extension("asm");

        fs::write(&output_path, output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_add() {
        let input = "push constant 7\npush constant 8\nadd";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("D=A"));
        assert!(result.contains("M=D+M"));
    }

    #[test]
    fn test_push_constant() {
        let input = "push constant 17";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("@17"));
        assert!(result.contains("D=A"));
    }

    #[test]
    fn test_pop_local() {
        let input = "pop local 0";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("@LCL"));
        assert!(result.contains("D=D+M"));
    }

    #[test]
    fn test_label() {
        let input = "label LOOP_START";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(LOOP_START)"));
    }

    #[test]
    fn test_goto() {
        let input = "goto END";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("@END"));
        assert!(result.contains("0;JMP"));
    }

    #[test]
    fn test_if_goto() {
        let input = "if-goto LOOP";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("@LOOP"));
        assert!(result.contains("D;JNE"));
    }

    #[test]
    fn test_simple_loop() {
        let input = r#"
push constant 0
pop local 0
label LOOP_START
push local 0
push constant 10
lt
if-goto LOOP_BODY
goto LOOP_END
label LOOP_BODY
push local 0
push constant 1
add
pop local 0
goto LOOP_START
label LOOP_END
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(LOOP_START)"));
        assert!(result.contains("(LOOP_BODY)"));
        assert!(result.contains("(LOOP_END)"));
        assert!(result.contains("@LOOP_START"));
        assert!(result.contains("@LOOP_BODY"));
        assert!(result.contains("@LOOP_END"));
    }

    #[test]
    fn test_conditional_branch() {
        let input = r#"
push constant 5
push constant 3
gt
if-goto TRUE_BRANCH
push constant 0
goto END
label TRUE_BRANCH
push constant 1
label END
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(TRUE_BRANCH)"));
        assert!(result.contains("(END)"));
        assert!(result.contains("D;JNE"));
    }

    #[test]
    fn test_nested_labels() {
        let input = r#"
label OUTER
push constant 5
label INNER
push constant 10
goto OUTER
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(OUTER)"));
        assert!(result.contains("(INNER)"));
    }

    #[test]
    fn test_label_with_valid_characters() {
        let input = r#"
label loop_start
label LOOP.END
label test:1
label _private
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(loop_start)"));
        assert!(result.contains("(LOOP.END)"));
        assert!(result.contains("(test:1)"));
        assert!(result.contains("(_private)"));
    }

    #[test]
    fn test_invalid_label_starts_with_digit() {
        let input = "label 123invalid";
        let result = VMTranslator::translate(input, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_label_empty() {
        let input = "label";
        let result = VMTranslator::translate(input, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_arithmetic_operations() {
        let input = r#"
push constant 10
push constant 5
sub
push constant 2
add
neg
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("M=M-D")); // sub
        assert!(result.contains("M=D+M")); // add
        assert!(result.contains("M=-M")); // neg
    }

    #[test]
    fn test_comparison_operations() {
        let input = r#"
push constant 5
push constant 3
eq
push constant 10
push constant 10
gt
push constant 2
push constant 8
lt
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("D;JEQ")); // eq
        assert!(result.contains("D;JGT")); // gt
        assert!(result.contains("D;JLT")); // lt
    }

    #[test]
    fn test_logical_operations() {
        let input = r#"
push constant 5
push constant 3
and
push constant 5
push constant 3
or
push constant 1
not
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("M=D&M")); // and
        assert!(result.contains("M=D|M")); // or
        assert!(result.contains("M=!M")); // not
    }

    #[test]
    fn test_all_segments() {
        let input = r#"
push constant 10
push local 0
push argument 1
push this 2
push that 3
push temp 5
push pointer 0
push pointer 1
pop local 0
pop argument 1
pop this 2
pop that 3
pop temp 5
pop pointer 0
pop pointer 1
"#;
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("@LCL"));
        assert!(result.contains("@ARG"));
        assert!(result.contains("@THIS"));
        assert!(result.contains("@THAT"));
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
