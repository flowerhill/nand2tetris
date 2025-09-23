use anyhow::{Context, Result};

use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("usage: {} <filename>", args[0]);
    }

    let input_file = &args[1];

    let assmbly_code = read_assembly(input_file)?;

    let code = preprocess(assmbly_code);

    let symbol_table = build_symbol_table(&code);

    let binary = assemble(&code, &symbol_table)?;

    let output_file = format!(
        "{}.hack",
        Path::new(input_file).file_stem().unwrap().to_str().unwrap()
    );

    write_binary_code(&output_file, binary)?;

    Ok(())
}

fn read_assembly(file_path: &str) -> Result<Vec<String>> {
    let file = File::open(file_path)?;

    let reader = BufReader::new(file);
    let mut lines = vec![];

    for line in reader.lines().map_while(Result::ok) {
        lines.push(line);
    }

    Ok(lines)
}

fn preprocess(assembly_code: Vec<String>) -> Vec<String> {
    assembly_code
        .iter()
        .filter_map(|line| {
            let code = if let Some(idx) = line.find("//") {
                &line[0..idx]
            } else {
                line
            };

            let trimmed = code.trim();

            // 空行をスキップ
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

fn build_symbol_table(code: &[String]) -> HashMap<String, u16> {
    // 初期化初期化
    let mut symbol_table = HashMap::new();

    symbol_table.insert(String::from("R0"), 0);
    symbol_table.insert(String::from("R1"), 1);
    symbol_table.insert(String::from("R2"), 2);
    symbol_table.insert(String::from("R3"), 3);
    symbol_table.insert(String::from("R4"), 4);
    symbol_table.insert(String::from("R5"), 5);
    symbol_table.insert(String::from("R6"), 6);
    symbol_table.insert(String::from("R7"), 7);
    symbol_table.insert(String::from("R8"), 8);
    symbol_table.insert(String::from("R9"), 9);
    symbol_table.insert(String::from("R10"), 10);
    symbol_table.insert(String::from("R11"), 11);
    symbol_table.insert(String::from("R12"), 12);
    symbol_table.insert(String::from("R13"), 13);
    symbol_table.insert(String::from("R14"), 14);
    symbol_table.insert(String::from("R15"), 15);

    symbol_table.insert(String::from("SP"), 0);
    symbol_table.insert(String::from("LCL"), 2);
    symbol_table.insert(String::from("ARG"), 3);
    symbol_table.insert(String::from("THIS"), 4);

    symbol_table.insert(String::from("SCREEN"), 16384);
    symbol_table.insert(String::from("KBD"), 24576);

    // 1回目のパス ラベルのみ処理
    let mut current_line_num = 0;
    for line in code {
        if line.starts_with('(') && line.ends_with(')') {
            let label = &line[1..line.len() - 1];
            symbol_table.insert(label.to_string(), current_line_num);
        } else {
            current_line_num += 1;
        }
    }

    // 2回目のパス 変数を処理
    let mut not_defined_variable = 16; // 未定義の変数は16から
    for line in code {
        if line.starts_with('@') && line[1..].parse::<u16>().is_err() {
            let symbol = &line[1..];
            if !symbol_table.contains_key(symbol) {
                symbol_table.insert(symbol.to_string(), not_defined_variable);
                not_defined_variable += 1;
            }
        }
    }

    symbol_table
}

fn assemble(code: &[String], symbol_table: &HashMap<String, u16>) -> Result<Vec<String>> {
    let mut binary_code = vec![];

    for line in code {
        if line.starts_with('(') && line.ends_with(')') {
            continue;
        }

        if let Some(sym) = line.strip_prefix('@') {
            // A命令
            let val = if let Ok(num) = sym.parse::<u16>() {
                // 数値
                num
            } else {
                // シンボル
                *symbol_table
                    .get(sym)
                    .with_context(|| format!("undefined symbol: {}", &sym[1..]))?
            };
            let binary = format!("{:016b}\n", val);
            binary_code.push(binary);
        } else {
            // C命令
            let parts: Vec<&str> = line.split(';').collect();

            let jump = if parts.len() > 1 {
                jump_table(parts[1]).to_string()
            } else {
                "000".to_string()
            };

            let dc_parts: Vec<&str> = parts[0].split('=').collect();

            let (dest, comp) = if dc_parts.len() > 1 {
                let dest_parts = dc_parts[0];
                let dest = format!(
                    "{}{}{}",
                    if dest_parts.contains('A') { "1" } else { "0" },
                    if dest_parts.contains('D') { "1" } else { "0" },
                    if dest_parts.contains('M') { "1" } else { "0" },
                );
                let comp = comp_table(dc_parts[1])?;
                (dest, comp.to_string())
            } else {
                let dest = String::from("000");
                let comp = comp_table(dc_parts[0])?;
                (dest, comp.to_string())
            };
            let binary = format!("111{}{}{}\n", comp, dest, jump);
            binary_code.push(binary);
        }
    }

    Ok(binary_code)
}

// compは必須のため、変換に失敗したらErrにする
fn comp_table(comp: &str) -> Result<&str> {
    match comp {
        // a = 0
        "0" => Ok("0101010"),
        "1" => Ok("0111111"),
        "-1" => Ok("0111010"),
        "D" => Ok("0001100"),
        "A" => Ok("0110000"),
        "!D" => Ok("0001101"),
        "!A" => Ok("0110001"),
        "-D" => Ok("0001111"),
        "-A" => Ok("0110011"),
        "D+1" => Ok("0011111"),
        "A+1" => Ok("0110111"),
        "D-1" => Ok("0001110"),
        "A-1" => Ok("0110010"),
        "D+A" => Ok("0000010"),
        "D-A" => Ok("0010011"),
        "A-D" => Ok("0000111"),
        "D&A" => Ok("0000000"),
        "D|A" => Ok("0010101"),
        // a = 1
        "M" => Ok("1110000"),
        "!M" => Ok("1110001"),
        "-M" => Ok("1110011"),
        "M+1" => Ok("1110111"),
        "M-1" => Ok("1110010"),
        "D+M" => Ok("1000010"),
        "D-M" => Ok("1010011"),
        "M-D" => Ok("1000111"),
        "D&M" => Ok("1000000"),
        "D|M" => Ok("1010101"),
        _ => anyhow::bail!("invalid comp pattern: {comp}"),
    }
}

fn jump_table(jump: &str) -> &str {
    match jump {
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111",
        _ => "000",
    }
}

fn write_binary_code(file_path: &str, binary_code: Vec<String>) -> Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    for line in binary_code {
        writer.write_all(line.as_bytes())?;
    }
    writer.flush()?;
    Ok(())
}
