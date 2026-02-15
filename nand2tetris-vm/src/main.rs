use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use std::{fs, path::Path};

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
            "return" => Ok(Command {
                command_type: CommandType::Return,
                arg1: None,
                arg2: None,
            }),
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
                self.pop_direct(&format!("{}.{}", self.filename, index));
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
        self.output.push("// call".to_string());

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

        self.output.push(format!("({return_address_symbol})"));

        self.call_counter += 1;
    }

    fn write_function(&mut self, function_name: &str, n_args: i32) {
        self.output.push("// function".to_string());

        self.output.push(format!("({})", function_name));

        for _ in 0..n_args {
            self.write_push("constant", 0);
        }
    }

    fn write_return(&mut self) {
        self.output.push("// return".to_string());

        // FRAME = LCL
        self.output.extend(vec![
            "@LCL".to_string(),
            "D=M".to_string(),
            "@13".to_string(),
            "M=D".to_string(),
        ]);

        // RET = *(FRAME - 5)
        self.output.extend(vec![
            "@5".to_string(),
            "A=D-A".to_string(),
            "D=M".to_string(),
            "@R14".to_string(),
            "M=D".to_string(),
        ]);

        // *ARG = pop()
        self.output.extend(vec![
            "@SP".to_string(),
            "M=M-1".to_string(),
            "A=M".to_string(),
            "D=M".to_string(),
            "@ARG".to_string(),
            "A=M".to_string(),
            "M=D".to_string(),
        ]);

        // SP = ARG + 1
        self.output.extend(vec![
            "@ARG".to_string(),
            "D=M+1".to_string(),
            "@SP".to_string(),
            "M=D".to_string(),
        ]);

        // THAT, THIS, ARG, LCL を復元
        for segment in ["THAT", "THIS", "ARG", "LCL"] {
            self.output.extend(vec![
                "@R13".to_string(),
                "AM=M-1".to_string(),
                "D=M".to_string(),
                format!("@{}", segment),
                "M=D".to_string(),
            ]);
        }

        // goto RET
        self.output.extend(vec![
            "@R14".to_string(),
            "A=M".to_string(),
            "0;JMP".to_string(),
        ]);
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
                CommandType::Call => {
                    let function_name = cmd.arg1.context("Missing function name")?;
                    let n_args = cmd.arg2.context("Missing function name")?;
                    code_writer.write_call(&function_name, n_args);
                }
                CommandType::Function => {
                    let function_name = cmd.arg1.context("Missing function name")?;
                    let n_args = cmd.arg2.context("Missing function name")?;
                    code_writer.write_function(&function_name, n_args);
                }
                CommandType::Return => code_writer.write_return(),
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
    use rstest::rstest;

    // ========================================
    // validate_label
    // ========================================

    #[rstest]
    #[case("LOOP")]
    #[case("_private")]
    #[case("test.label")]
    #[case("foo:bar")]
    #[case("a1b2c3")]
    #[case("LOOP_START")]
    #[case("LOOP.END")]
    #[case("test:1")]
    fn test_validate_label_ok(#[case] label: &str) {
        assert!(validate_label(label).is_ok());
    }

    #[rstest]
    #[case("")]
    #[case("123abc")]
    #[case("123invalid")]
    #[case("@invalid")]
    #[case("hello world")]
    #[case("-start")]
    fn test_validate_label_err(#[case] label: &str) {
        assert!(validate_label(label).is_err());
    }

    // ========================================
    // Parser: コメント・空行・空入力
    // ========================================

    #[rstest]
    #[case("// comment\npush constant 5 // inline\n// end", "@5")]
    #[case("\n\n\npush constant 42\n\n\n", "@42")]
    fn test_parser_filters_non_code(#[case] input: &str, #[case] expected: &str) {
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains(expected));
    }

    #[rstest]
    #[case("// just comments\n// another")]
    #[case("")]
    fn test_empty_output(#[case] input: &str) {
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.is_empty());
    }

    // ========================================
    // Parser 単体
    // ========================================

    #[test]
    fn test_parser_return_command() {
        let parser = Parser::new("return");
        let cmd = parser.parse().unwrap();
        assert_eq!(cmd.command_type, CommandType::Return);
        assert!(cmd.arg1.is_none());
        assert!(cmd.arg2.is_none());
    }

    #[test]
    fn test_parser_advance_and_bounds() {
        let mut parser = Parser::new("push constant 1\npush constant 2\npush constant 3");
        assert!(parser.has_more_commands());
        assert_eq!(parser.current_line_number(), 1);
        parser.advance();
        assert_eq!(parser.current_line_number(), 2);
        parser.advance();
        assert_eq!(parser.current_line_number(), 3);
        parser.advance();
        assert!(!parser.has_more_commands());
        parser.advance(); // 超過しても panic しない
        assert!(!parser.has_more_commands());
    }

    // ========================================
    // エラーケース
    // ========================================

    #[rstest]
    #[case("foobar")]
    #[case("push")]
    #[case("push constant")]
    #[case("push constant abc")]
    #[case("pop")]
    #[case("pop local")]
    #[case("goto")]
    #[case("if-goto")]
    #[case("call")]
    #[case("call Foo.bar")]
    #[case("call Foo.bar xyz")]
    #[case("function")]
    #[case("function Foo.bar")]
    #[case("label")]
    #[case("label @invalid")]
    #[case("label 123invalid")]
    fn test_invalid_input(#[case] input: &str) {
        assert!(VMTranslator::translate(input, "test").is_err());
    }

    // ========================================
    // push セグメント
    // ========================================

    #[rstest]
    #[case("push constant 17",  "test",   &["@17", "D=A"])]
    #[case("push constant 100", "test",   &["@100", "D=A"])]
    #[case("push local 0",      "test",   &["@LCL"])]
    #[case("push argument 1",   "test",   &["@ARG"])]
    #[case("push this 2",       "test",   &["@THIS"])]
    #[case("push that 3",       "test",   &["@THAT"])]
    #[case("push temp 2",       "test",   &["@7"])]
    #[case("push temp 5",       "test",   &["@10"])]
    #[case("push pointer 0",    "test",   &["@THIS", "D=M"])]
    #[case("push pointer 1",    "test",   &["@THAT", "D=M"])]
    #[case("push static 3",     "MyFile", &["@MyFile.3"])]
    #[case("push static 0",     "Foo",    &["@Foo.0"])]
    #[case("push static 0",     "Bar",    &["@Bar.0"])]
    fn test_push(#[case] input: &str, #[case] filename: &str, #[case] expected: &[&str]) {
        let result = VMTranslator::translate(input, filename).unwrap();
        for s in expected {
            assert!(
                result.contains(s),
                "Expected '{}' in output for '{}'",
                s,
                input
            );
        }
    }

    // ========================================
    // pop セグメント
    // ========================================

    #[rstest]
    #[case("pop local 0",    "test",   &["@LCL", "D=D+M"])]
    #[case("pop argument 1", "test",   &["@ARG"])]
    #[case("pop this 2",     "test",   &["@THIS"])]
    #[case("pop that 3",     "test",   &["@THAT"])]
    #[case("pop temp 0",     "test",   &["@5"])]
    #[case("pop pointer 0",  "test",   &["@THIS"])]
    #[case("pop pointer 1",  "test",   &["@THAT"])]
    fn test_pop(#[case] input: &str, #[case] filename: &str, #[case] expected: &[&str]) {
        let result = VMTranslator::translate(input, filename).unwrap();
        for s in expected {
            assert!(
                result.contains(s),
                "Expected '{}' in output for '{}'",
                s,
                input
            );
        }
    }

    // ========================================
    // 算術・論理
    // ========================================

    #[rstest]
    #[case("add", "M=D+M")]
    #[case("sub", "M=M-D")]
    #[case("neg", "M=-M")]
    #[case("and", "M=D&M")]
    #[case("or", "M=D|M")]
    #[case("not", "M=!M")]
    fn test_arithmetic(#[case] op: &str, #[case] expected: &str) {
        let input = format!("push constant 3\npush constant 5\n{}", op);
        let result = VMTranslator::translate(&input, "test").unwrap();
        assert!(result.contains(expected));
    }

    // ========================================
    // 比較
    // ========================================

    #[rstest]
    #[case("eq", "D;JEQ")]
    #[case("gt", "D;JGT")]
    #[case("lt", "D;JLT")]
    fn test_comparison(#[case] op: &str, #[case] expected_jump: &str) {
        let input = format!("push constant 3\npush constant 5\n{}", op);
        let result = VMTranslator::translate(&input, "test").unwrap();
        assert!(result.contains(expected_jump));
        assert!(result.contains("(TRUE_0)"));
        assert!(result.contains("(END_0)"));
    }

    #[test]
    fn test_multiple_comparisons_unique_labels() {
        let input = "push constant 1\npush constant 2\neq\n\
                      push constant 3\npush constant 4\ngt\n\
                      push constant 5\npush constant 6\nlt";
        let result = VMTranslator::translate(input, "test").unwrap();
        for i in 0..3 {
            assert!(result.contains(&format!("(TRUE_{})", i)));
            assert!(result.contains(&format!("(END_{})", i)));
        }
    }

    // ========================================
    // label / goto / if-goto
    // ========================================

    #[test]
    fn test_label_goto_if_goto() {
        let input = "label LOOP\ngoto END\nif-goto LOOP";
        let result = VMTranslator::translate(input, "test").unwrap();
        for s in ["(LOOP)", "@END", "0;JMP", "@LOOP", "D;JNE"] {
            assert!(result.contains(s));
        }
    }

    #[rstest]
    #[case("label loop_start", "(loop_start)")]
    #[case("label LOOP.END", "(LOOP.END)")]
    #[case("label test:1", "(test:1)")]
    #[case("label _private", "(_private)")]
    fn test_label_valid_chars(#[case] input: &str, #[case] expected: &str) {
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains(expected));
    }

    // ========================================
    // call
    // ========================================

    #[test]
    fn test_call() {
        let result = VMTranslator::translate("call Foo.bar 3", "test").unwrap();
        for s in [
            "Foo.bar$ret0",
            "@LCL",
            "@ARG",
            "@THIS",
            "@THAT",
            "@8",
            "@Foo.bar",
            "0;JMP",
        ] {
            assert!(result.contains(s), "Expected '{}'", s);
        }
    }

    // ========================================
    // function
    // ========================================

    #[test]
    fn test_function() {
        let result = VMTranslator::translate("function Foo.bar 2", "test").unwrap();
        assert!(result.contains("(Foo.bar)"));
        assert!(result.contains("@0"));
    }

    // ========================================
    // return
    // ========================================

    #[test]
    fn test_return() {
        let result = VMTranslator::translate("return", "test").unwrap();
        for s in ["@LCL", "@13", "@R14", "@5", "AM=M-1", "@ARG", "0;JMP"] {
            assert!(result.contains(s), "Expected '{}'", s);
        }
    }

    // ========================================
    // 統合テスト
    // ========================================

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
        for s in [
            "(LOOP_START)",
            "(LOOP_BODY)",
            "(LOOP_END)",
            "@LOOP_START",
            "@LOOP_BODY",
            "@LOOP_END",
        ] {
            assert!(result.contains(s));
        }
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
        for s in ["(TRUE_BRANCH)", "(END)", "D;JNE"] {
            assert!(result.contains(s));
        }
    }

    #[test]
    fn test_nested_labels() {
        let input = "label OUTER\npush constant 5\nlabel INNER\npush constant 10\ngoto OUTER";
        let result = VMTranslator::translate(input, "test").unwrap();
        assert!(result.contains("(OUTER)"));
        assert!(result.contains("(INNER)"));
    }

    #[test]
    fn test_multiple_arithmetic_operations() {
        let input = "push constant 10\npush constant 5\nsub\npush constant 2\nadd\nneg";
        let result = VMTranslator::translate(input, "test").unwrap();
        for s in ["M=M-D", "M=D+M", "M=-M"] {
            assert!(result.contains(s));
        }
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
        for s in ["@LCL", "@ARG", "@THIS", "@THAT"] {
            assert!(result.contains(s));
        }
    }

    #[test]
    fn test_function_call_return_integration() {
        let input = "function Main.main 0\npush constant 3\ncall Math.mul 1\nreturn\n\
                      function Math.mul 1\npush argument 0\npop local 0\npush local 0\nreturn";
        let result = VMTranslator::translate(input, "test").unwrap();
        for s in ["(Main.main)", "(Math.mul)", "Math.mul$ret", "@R14"] {
            assert!(result.contains(s));
        }
    }

    #[test]
    fn test_fibonacci_like_loop() {
        let input = "push constant 0\npop local 0\npush constant 1\npop local 1\n\
                      label LOOP\npush local 0\npush local 1\nadd\npop local 1\npop local 0\n\
                      push local 1\npush constant 100\nlt\nif-goto LOOP";
        let result = VMTranslator::translate(input, "test").unwrap();
        for s in ["(LOOP)", "@LOOP", "D;JNE", "M=D+M"] {
            assert!(result.contains(s));
        }
    }
}
