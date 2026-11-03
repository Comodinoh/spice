use std::collections::HashMap;
use std::fs::File;

#[derive(Clone, Debug)]
pub enum LexerError<'a> {
    UnknownToken(&'a str, usize, usize, String),
    DigitAtIdentifierStart(&'a str, usize, usize, String),
    UnexpectedEOF(&'a str, usize, usize),
}

#[derive(Debug, PartialEq)]
pub enum KeywordKind {
    Int,
    Return,
    Let,
    If,
    Else,
    Proc,
    For,
}

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
    Plus,
    Minus,
    Star,
    Slash,
    Not,
    Equality,
    Inequality,
    Greater,
    Lesser,
    GreaterOrEqual,
    LesserOrEqual,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    IntegerLiteral,
    Keyword(KeywordKind),
    Operator(OperatorKind),
    OpenBracket,
    ClosedBracket,
    Semicolon,
    Comma,
    OpenParen,
    ClosedParen,
    Identifier,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Token<'a> {
    kind: TokenKind,
    line: usize,
    column: usize,
    expr: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, line: usize, column: usize, expr: &'a str) -> Self {
        Self {
            kind,
            line,
            column,
            expr,
        }
    }
}

trait Operator {
    fn get_operator_kind(&self) -> Option<OperatorKind>;
}

trait ConditionalOperator {
    fn is_conditional_operator(&self) -> bool;
}

trait Keyword {
    fn get_keyword_kind(&self) -> Option<KeywordKind>;
}

trait Identifier {
    fn is_identifier_first(&self) -> bool;
    fn is_identifier_continue(&self) -> bool;
}

trait Number {
    fn is_number(&self) -> bool;
}

impl Number for str {
    fn is_number(&self) -> bool {
        for c in self.chars() {
            if !c.is_ascii_digit() && c != '_' {
                return false;
            }
        }
        return true;
    }
}

impl Operator for str {
    fn get_operator_kind(&self) -> Option<OperatorKind> {
        match self {
            "+" => Some(OperatorKind::Plus),
            "-" => Some(OperatorKind::Minus),
            "*" => Some(OperatorKind::Star),
            "/" => Some(OperatorKind::Slash),
            "==" => Some(OperatorKind::Equality),
            "!=" => Some(OperatorKind::Inequality),
            ">" => Some(OperatorKind::Greater),
            ">=" => Some(OperatorKind::GreaterOrEqual),
            "<" => Some(OperatorKind::Lesser),
            "<=" => Some(OperatorKind::LesserOrEqual),
            "!" => Some(OperatorKind::Not),
            _ => None
        }
    }
    // fn is_operator(&self) -> bool {
    //     match self.as_str() {
    //         "+" | "-" | "*" | "/" | "^" | "%" | "&" | "|"  => true,
    //         _ => false,
    //     }
    // }
}

impl Keyword for str {
    fn get_keyword_kind(&self) -> Option<KeywordKind> {
        match self {
            "int" => Some(KeywordKind::Int),
            "return" => Some(KeywordKind::Return),
            "for" => Some(KeywordKind::For),
            "let" => Some(KeywordKind::Let),
            "if" => Some(KeywordKind::If),
            "else" => Some(KeywordKind::Else),
            "proc" => Some(KeywordKind::Proc),
            _ => None,
        }
    }
}

impl Identifier for char {
    fn is_identifier_first(&self) -> bool {
        matches!(self, 'a'..'z' | 'A'..'Z' | '_')
    }

    fn is_identifier_continue(&self) -> bool {
        matches!(self, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
    }
}

pub struct Lexer<'a> {
    pos: usize,
    current_source: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new() -> Self {
        Self {pos: 0, current_source: ""}
    }

    pub fn peek(&self, n: isize) -> Option<char> {
        self.current_source.as_bytes().get(self.pos.checked_add_signed(n)?).map(|x| *x as char)
    }

    pub fn take(&mut self, n: isize) -> Option<char> {
        let peeked = self.peek(n)?;
        self.seek(1);

        Some(peeked)
    }

    pub fn eat(&mut self, expected: char) -> bool {
        if self.peek(0) == Some(expected) { self.seek(1); true } else { false }
    }

    pub fn seek(&mut self, n: isize) -> Option<usize>{
        self.pos = self.pos.checked_add_signed(n)?;

        return Some(self.pos);
    }


    pub fn consume(&mut self, n: isize) -> Option<&'a str>{
        let start = self.pos;
        let mut end = self.pos;

        if let None = self.peek(0) {
            return None;
        }

        for i in 0..n {
            if self.peek(i).is_none() {
                break;
            }

            end += 1;
        }

        self.seek((end - start) as isize);

        Some(&self.current_source[start..end])
    }

    pub fn consume_if(&mut self, filter: fn(c: char) -> bool) -> Option<&'a str> {
        let start = self.pos;
        let mut end = self.pos;

        if self.peek(0).is_none() {
            return None
        }
        
        while let Some(current) = self.peek(0) && filter(current) {
            end += 1;
            self.seek(1);
        }


        Some(&self.current_source[start..end])
    }

    pub fn tokenize(&mut self, src: &'a str) -> Result<Vec<Token<'a>>, LexerError<'a>> {
        let mut result = vec![];

        self.pos = 0;
        self.current_source = src;

        let mut line = 1_usize;
        let mut line_start = 0_usize;

        while self.pos < src.len() {
            let current = match self.peek(0) {
                Some(p) => p,
                None => return Ok(result)
            };
            
            if current == '\n' {
                self.seek(1);

                line += 1;
                line_start = self.pos;
                
                continue;
            }

            if current.is_whitespace() {
                self.seek(1);
                continue;
            }

            let single_tok = match current {
                '{' => Some(TokenKind::OpenBracket),
                '}' => Some(TokenKind::ClosedBracket),
                '(' => Some(TokenKind::OpenParen),
                ')' => Some(TokenKind::ClosedParen),
                ';' => Some(TokenKind::Semicolon),
                ',' => Some(TokenKind::Comma),
                _ => None,
            };

            if let Some(kind) = single_tok {
                let start = self.pos;
                self.seek(1);

                result.push(Token::new(
                    kind,
                    line,
                    start - line_start + 1,
                    &self.current_source[start..self.pos],
                ));

                continue;
            }

            let expr;

            if current.is_ascii_digit() {
                let start = self.pos;
                expr = self.consume_if(|c|
                        !c.is_whitespace() && (c.is_ascii_digit() || c.is_identifier_first())
                ).unwrap_or_else(|| "");

                let column = start - line_start + 1;

                if !expr.is_number() {
                    return Err(LexerError::DigitAtIdentifierStart(self.current_source, line, column, String::from(expr)));
                }

                result.push(Token::new(
                    TokenKind::IntegerLiteral,
                    line,
                    column,
                    expr,
                ));

                continue;
            }

            if current.is_identifier_first() {
                let start = self.pos;

                expr = self.consume_if(|c|
                    !c.is_whitespace() && c.is_identifier_continue()
                ).unwrap_or_else(|| "");

                let column = start - line_start + 1;

                let keyword_kind = expr.get_keyword_kind();
                if let Some(keyword_kind) = keyword_kind {

                    result.push(Token::new(
                        TokenKind::Keyword(keyword_kind),
                        line,
                        column,
                        expr,
                    ));

                    continue;
                }


                if !expr.is_empty() {
                    result.push(Token::new(
                        TokenKind::Identifier,
                        line,
                        column,
                        expr,
                    ));

                    continue;
                }

                return Err(LexerError::UnknownToken(self.current_source, line, start - line_start + 1, String::from(expr)));
            }

            if let Some(expr) = self.consume(2) {
                let kind = expr.get_operator_kind();

                if let Some(kind) = kind {
                    result.push(Token::new(
                        TokenKind::Operator(kind),
                        line,
                        (self.pos - 2) - line_start + 1,
                        expr
                    ));

                    continue;
                }

                self.seek(-2);

                if let Some(expr) = self.consume(1) {
                    let kind = expr.get_operator_kind();

                    if let Some(kind) = kind {
                        result.push(Token::new(
                            TokenKind::Operator(kind),
                            line,
                            (self.pos - 1) - line_start + 1,
                            expr
                        ));

                        continue;
                    }
                }

                self.seek(-1);
            }

            self.seek(1);
        }

        Ok(result)
    }
}

pub fn panic_lexer_err(err: LexerError) -> !{
    match err {
        LexerError::UnknownToken(source, line, column, expr) => {
            panic!("Unknown token {expr} at line {line} column {column} of original input:\n{}", source);
        },
        LexerError::DigitAtIdentifierStart(source, line, column, expr) => {
            panic!("Identifiers like {expr} cannot start with digits at line {line} column {column} of original input:\n{}", source);
        },
        LexerError::UnexpectedEOF(source, line, column) => {
            panic!("Unexpected EOF encountered at line {line} column {column} of original input:\n{}", source);
        }
    }
}

pub fn print_err(err: LexerError) -> !{
    eprintln!("ERROR: {err:?}");
    std::process::exit(1);
}

struct Parser {
    
}


fn main() {
    let mut lexer = Lexer::new();

    let args = std::env::args();
    let args: Vec<String> = args.filter(|arg| !arg.starts_with('-'))
        .collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <files>", args[0]);

        std::process::exit(1);
    }

    let filenames = args[1..].to_vec();
    let mut file_contents: HashMap<String, String> = HashMap::with_capacity(filenames.len() + 1);

    for filename in filenames {
        file_contents.insert(filename.clone(), match std::fs::read_to_string(&filename) {
            Ok(result) => result,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {eprintln!("File {filename} not found"); return}
                _ => panic!("Error opening file {filename}: {e}")
            }
        });
    }

    for (name, content) in &file_contents {
        let tokens = lexer.tokenize(&content)
            .unwrap_or_else(|e| print_err(e));
        println!("{name}: {tokens:#?}");
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_punctuation() {
        let mut lexer = Lexer::new();

        let tokens = lexer.tokenize("(()) {{
                ,;
         }}").unwrap_or_else(|e| panic_lexer_err(e));

        let expected_tokens = vec![
            Token::new(TokenKind::OpenParen,1, 1,"("),
            Token::new(TokenKind::OpenParen,1, 2,"("),
            Token::new(TokenKind::ClosedParen,1, 3,")"),
            Token::new(TokenKind::ClosedParen,1, 4,")"),
            Token::new(TokenKind::OpenBracket,1, 6,"{"),
            Token::new(TokenKind::OpenBracket,1, 7,"{"),

            Token::new(TokenKind::Comma,2, 17,","),
            Token::new(TokenKind::Semicolon,2, 18,";"),

            Token::new(TokenKind::ClosedBracket,3, 10,"}"),
            Token::new(TokenKind::ClosedBracket,3, 11,"}"),
        ];

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = Lexer::new();

        let tokens = lexer.tokenize("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_ ABCDEFGHIJKLMNOPQRSTUVWXYabcdefghijklmnopqrstuvwxyzZ1234567890_ _ABCDEFGHIJKLMNOPQRSTUVWXYabcdefghijklmnopqrstuvwxyzZ1234567890 _abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890").unwrap_or_else(|e| panic_lexer_err(e));

        let expected_tokens = vec![
            Token::new(TokenKind::Identifier,1, 1,"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"),
            Token::new(TokenKind::Identifier,1, 65,"ABCDEFGHIJKLMNOPQRSTUVWXYabcdefghijklmnopqrstuvwxyzZ1234567890_"),
            Token::new(TokenKind::Identifier,1, 129,"_ABCDEFGHIJKLMNOPQRSTUVWXYabcdefghijklmnopqrstuvwxyzZ1234567890"),
            Token::new(TokenKind::Identifier,1, 193,"_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890"),
        ];

        assert_eq!(tokens, expected_tokens);

        lexer.tokenize("1234567890abcdfghhijklm_").unwrap_err();
    }

    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new();

        let tokens = lexer.tokenize("1234567890 123456_7890").unwrap_or_else(|e| panic_lexer_err(e));

        let expected_tokens = vec![
            Token::new(TokenKind::IntegerLiteral,1, 1,"1234567890"),
            Token::new(TokenKind::IntegerLiteral,1, 12,"123456_7890"),
        ];

        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new();

        let tokens = lexer.tokenize("+-*/==!=<><=>=").unwrap_or_else(|e| panic_lexer_err(e));

        let expected_tokens = vec![
            Token::new(TokenKind::Operator(OperatorKind::Plus),1, 1,"+"),
            Token::new(TokenKind::Operator(OperatorKind::Minus),1, 2,"-"),
            Token::new(TokenKind::Operator(OperatorKind::Star),1, 3,"*"),
            Token::new(TokenKind::Operator(OperatorKind::Slash),1, 4,"/"),
            Token::new(TokenKind::Operator(OperatorKind::Equality),1, 5,"=="),
            Token::new(TokenKind::Operator(OperatorKind::Inequality),1, 7,"!="),
            Token::new(TokenKind::Operator(OperatorKind::Lesser),1, 9,"<"),
            Token::new(TokenKind::Operator(OperatorKind::Greater),1, 10,">"),
            Token::new(TokenKind::Operator(OperatorKind::LesserOrEqual),1, 11,"<="),
            Token::new(TokenKind::Operator(OperatorKind::GreaterOrEqual),1, 13,">="),

        ];

        assert_eq!(tokens, expected_tokens);
    }

}
