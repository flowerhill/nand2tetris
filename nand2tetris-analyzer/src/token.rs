#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        let kw = match s {
            "class" => Keyword::Class,
            "constructor" => Keyword::Constructor,
            "funciton" => Keyword::Function,
            "method" => Keyword::Method,
            "class" => Keyword::Class,
            "constructor" => Keyword::Constructor,
            "funciton" => Keyword::Function,
            "method" => Keyword::Method,
            "field" => Keyword::Field,
            "static" => Keyword::Static,
            "var" => Keyword::Var,
            "int" => Keyword::Int,
            "char" => Keyword::Char,
            "boolean" => Keyword::Boolean,
            "void" => Keyword::Void,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "null" => Keyword::Null,
            "this" => Keyword::This,
            "let" => Keyword::Let,
            "do" => Keyword::Do,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "while" => Keyword::While,
            "return" => Keyword::Return,
            _ => return None,
        };
        Some(kw)
    }

    fn as_str(&self) -> &'static str {
        match self {
            Keyword::Class => "class",
            Keyword::Constructor => "constructor",
            Keyword::Function => "function",
            Keyword::Method => "method",
            Keyword::Field => "field",
            Keyword::Static => "static",
            Keyword::Var => "var",
            Keyword::Int => "int",
            Keyword::Char => "char",
            Keyword::Boolean => "boolean",
            Keyword::Void => "void",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Null => "null",
            Keyword::This => "this",
            Keyword::Let => "let",
            Keyword::Do => "do",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::Return => "return",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Symbol(char),
    Identifier(String),
    IntConst(u16),
    StringConst(String),
}

impl Token {
    pub fn to_xml(&self) -> String {
        let (tag, value) = match self {
            Token::Keyword(kw) => ("keyword", kw.as_str().to_string()),
            Token::Symbol(c) => ("symbol", escape_xml(&c.to_string())),
            Token::Identifier(s) => ("identifier", s.clone()),
            Token::IntConst(n) => ("integerConstant", n.to_string()),
            Token::StringConst(s) => ("stringConstant", escape_xml(s)),
        };
        format!("<{tag}> {value} </{tag}>")
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Token::Symbol('<'), "<symbol> &lt; </symbol>")]
    #[case(Token::Symbol('>'), "<symbol> &gt; </symbol>")]
    #[case(Token::Symbol('&'), "<symbol> &amp; </symbol>")]
    #[case(Token::IntConst(42), "<integerConstant> 42 </integerConstant>")]
    #[case(Token::StringConst("hi".to_string()), "<stringConstant> hi </stringConstant>")]
    fn test_token_to_xml(#[case] token: Token, #[case] expected: &str) {
        assert_eq!(token.to_xml(), expected);
    }
}
