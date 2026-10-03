use anyhow::{bail, ensure, Context, Result};
use itertools::Itertools;

use crate::token::{Keyword, Token};

const SYMBOLS: &str = "{}()[].,;+-*/&|<>=~";

pub struct JackTokenizer {
    tokens: Vec<Token>,
    current: usize,
}

impl JackTokenizer {
    pub fn new(input: &str) -> Result<Self> {
        let tokens = Self::tokenize(input)?;
        Ok(JackTokenizer { tokens, current: 0 })
    }

    fn tokenize(input: &str) -> Result<Vec<Token>> {
        let chars: Vec<char> = input.chars().collect_vec();
        let mut tokens = vec![];
        let mut line = 1;
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // 空白
            if c.is_whitespace() {
                if c.is_whitespace() {
                    if c == '\n' {
                        line += 1;
                    }
                }
                i += 1;
                continue;
            }

            // 行コメント: // ... 改行
            if c == '/' && chars.get(i + 1) == Some(&'/') {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            if c == '/' && chars.get(i + 1) == Some(&'*') {
                i += 2;
                loop {
                    ensure!(i + 1 < chars.len(), "Line {}: unterminated comment", line);
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
                continue;
            }

            if SYMBOLS.contains(c) {
                tokens.push(Token::Symbol(c));
                i += 1;
                continue;
            }

            // 文字列定数(改行と"は含められない)
            if c == '"' {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' {
                    ensure!(
                        chars[i] != '\n',
                        "Line {}: newline in string constant",
                        line
                    );
                    i += 1;
                }
                ensure!(
                    i < chars.len(),
                    "Line {}: unterminated string constant",
                    line
                );
                tokens.push(Token::StringConst(chars[start..i].iter().collect()));
                i += 1; // 閉じ " を読み飛ばす
                continue;
            }

            if c.is_ascii_digit() {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let n: u16 = s
                    .parse()
                    .ok()
                    .filter(|n| *n <= 32767)
                    .context(format!("Line {}: integer out of range: {}", line, s))?;
                tokens.push(Token::IntConst(n));
                continue;
            }

            if c.is_ascii_alphabetic() || c == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match Keyword::from_str(&word) {
                    Some(kw) => tokens.push(Token::Keyword(kw)),
                    None => tokens.push(Token::Identifier(word)),
                }
                continue;
            }

            bail!("Line {}: unexpected character '{}'", line, c);
        }
        Ok(tokens)
    }

    pub fn has_more_tokens(&self) -> bool {
        self.current < self.tokens.len()
    }

    pub fn advance(&mut self) {
        if self.has_more_tokens() {
            self.current += 1;
        }
    }

    pub fn token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    pub fn to_xml(&self) -> String {
        let mut lines = vec!["<tokens>".to_string()];
        for token in &self.tokens {
            lines.push(token.to_xml());
        }
        lines.push("</tokens>".to_string());
        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn tokenize(input: &str) -> Vec<Token> {
        JackTokenizer::tokenize(input).unwrap()
    }

    #[test]
    fn test_tokenize_comments_are_skipped() {
        let input = "// input\n/* block */ /** doc\n * comment */ let";
        assert_eq!(tokenize(input), vec![Token::Keyword(Keyword::Let)]);
    }

    #[test]
    fn test_tokenize_division_is_not_comment() {
        assert_eq!(
            tokenize("a/b"),
            vec![
                Token::Identifier("a".to_string()),
                Token::Symbol('/'),
                Token::Identifier("b".to_string())
            ]
        )
    }

    #[test]
    fn test_tokenize_string_keeps_comment_like_text() {
        assert_eq!(
            tokenize("\"a // b\""),
            vec![Token::StringConst("a // b".to_string()),]
        )
    }

    #[test]
    fn test_tokenize_keyword_prefix_is_identifier() {
        assert_eq!(
            tokenize("classy"),
            vec![Token::Identifier("classy".to_string())]
        );
    }

    #[rstest]
    #[case("32768")]
    #[case("\"abc")]
    #[case("\"a\nb\"")]
    #[case("/* never closed")]
    #[case("#")]
    fn test_tokenize_err(#[case] input: &str) {
        assert!(JackTokenizer::tokenize(input).is_err());
    }
}
