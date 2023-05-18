use crate::tokenizer::{TokenInfo, Token, Position};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    Generic(TokenInfo, String),
    InvalidFor(TokenInfo),
    InvalidAssignment(TokenInfo, String),
    MissingClosingBrackets(TokenInfo),
    MissingClosingParantheses(TokenInfo),
    ExpectedStartingBrackets(TokenInfo),
    ExpectedStartingParantheses(TokenInfo),
    MissingSemicolon(TokenInfo),
    UndefinedVariable(TokenInfo)
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(token_info, string) =>
                write!(f, "Syntax error: unexpected token '{}' of type {} after {} on line {}", token_info.lexeme, token_info.token.to_string(), string, token_info.start_position.row),
            Error::InvalidFor(token_info) =>
                write!(f, "Syntax error: invalid for loop structure, unexpected token '{}' of type {} on line {}", token_info.lexeme, token_info.token.to_string(), token_info.start_position.row),
            Error::InvalidAssignment(token_info, string) =>
                write!(f, "Syntax error: invalid assignment; found '{}' of type {} after {} on line {}", token_info.lexeme, token_info.token.to_string(), string, token_info.start_position.row),
            Error::MissingClosingBrackets(token_info) =>
                write!(f, "Syntax error: missing closing brackets on line {}", token_info.start_position.row),
            Error::MissingClosingParantheses(token_info) =>
                write!(f, "Syntax error: missing closing parantheses on line {}", token_info.start_position.row),
            Error::ExpectedStartingBrackets(token_info) =>
                write!(f, "Syntax error: expected {{, found '{}' on line {}", token_info.lexeme, token_info.start_position.row),
            Error::ExpectedStartingParantheses(token_info) =>
                write!(f, "Syntax error: expected (, found '{}' on line {}", token_info.lexeme, token_info.start_position.row),
            Error::MissingSemicolon(token_info) =>
                write!(f, "Syntax error: missing semicolon ';' on line {}", token_info.start_position.row),
            Error::UndefinedVariable(token_info) =>
                write!(f, "Evaluation error: variable '{}' on line {} undefined", token_info.lexeme, token_info.start_position.row)
        }
    }
}

struct ParserInfo<'slice> {
    tokens: &'slice [TokenInfo],
    current_token_info: TokenInfo,
    i: usize,
    variables: &'slice mut HashMap<String, i64>
}

impl ParserInfo<'_> {
    fn match_token(&mut self, expected_token: Token) -> bool {
        self.current_token_info = self.tokens[self.i].clone();
        if self.tokens[self.i].token == expected_token {
            self.i += 1;
            return true;
        }

        false
    }

    fn last_n_token_lexemes(&self, n: u32) -> String {
        let mut counter = 1;
        let mut string: String = String::from("");
        while n > 0 {
            string = format!("{} {}", &string, self.tokens[self.i - counter].lexeme);
            counter += 1;

            if self.i - counter == 0 {
                break;
            }
        }

        string.chars().rev().collect::<String>()
    }

    fn evaluate_bitwise(&mut self) -> Result<i64, Error> {
        let mut value = self.evaluate_additive()?;
        while self.match_token(Token::BWAnd) || self.match_token(Token::BWOr) {
            let operator = self.current_token_info.token;
            let next_value = self.evaluate_additive()?;
            match operator {
                Token::BWAnd => value = value & next_value,
                Token::BWOr => value = value | next_value,
                _ => return Err(Error::Generic(self.current_token_info.clone(), self.last_n_token_lexemes(3))),
            }
        }
        Ok(value)
    }

    fn evaluate_additive(&mut self) -> Result<i64, Error> {
        let mut value = self.evaluate_multiplicative()?;
        while self.match_token(Token::Addition) || self.match_token(Token::Subtraction) {
            let operator = self.current_token_info.token;
            let next_value = self.evaluate_multiplicative()?;
            match operator {
                Token::Addition => value = value + next_value,
                Token::Subtraction => value = value - next_value,
                _ => return Err(Error::Generic(self.current_token_info.clone(), self.last_n_token_lexemes(3))),
            }
        }
        Ok(value)
    }

    fn evaluate_multiplicative(&mut self) -> Result<i64, Error> {
        let mut value = self.evaluate_unary()?;
        while self.match_token(Token::Multiplication) || self.match_token(Token::Division) {
            let operator = self.current_token_info.token;
            let next_value = self.evaluate_unary()?;
            match operator {
                Token::Multiplication => value = value * next_value,
                Token::Division => value = value / next_value,
                _ => return Err(Error::Generic(self.current_token_info.clone(), self.last_n_token_lexemes(3))),
            }
        }
        Ok(value)
    }

    fn evaluate_unary(&mut self) -> Result<i64, Error> {
        if self.match_token(Token::Addition) {
            return self.evaluate_primary();
        } else if self.match_token(Token::Subtraction) {
            let value = self.evaluate_primary()?;
            return Ok(-value);
        }
        self.evaluate_primary()
    }

    fn evaluate_primary(&mut self) -> Result<i64, Error> {
        if self.match_token(Token::Int) {
            Ok(self.current_token_info.lexeme.parse().unwrap())
        } else if self.match_token(Token::Hex) {
            let hex_value = self.current_token_info.lexeme.trim_start_matches("#");
            Ok(i64::from_str_radix(hex_value, 16).unwrap())
        } else if self.match_token(Token::Identifier) {
            let var = self.current_token_info.clone();
            if self.match_token(Token::Assignment) {
                let value = self.evaluate_bitwise()?;
                self.variables.insert(var.lexeme, value);
                println!("{:?}", self.variables);
                Ok(value)
            } else {
                match self.variables.get(&var.lexeme) {
                    Some(value) => Ok(*value),
                    None => Err(Error::UndefinedVariable(var)),
                }
            }
        } else if self.match_token(Token::Console) {
            self.evaluate_bitwise()
        } else if self.match_token(Token::LeftParantheses) {
            let value = self.evaluate_bitwise()?;
            if !self.match_token(Token::RightParantheses) {
                return Err(Error::MissingClosingParantheses(self.current_token_info.clone()));
            }
            Ok(value)
        }
        else if self.match_token(Token::For) {
            self.evaluate_for()
        } else {
            Err(Error::Generic(self.current_token_info.clone(), self.last_n_token_lexemes(3)))
        }
    }

    fn evaluate_for(&mut self) -> Result<i64, Error> {
        self.match_token(Token::LeftParantheses);
        self.match_token(Token::Identifier);
        let var = self.current_token_info.lexeme.clone();
        self.match_token(Token::Assignment);

        let eval = self.evaluate_bitwise()?;
        self.variables.insert(var.clone(), eval);
        self.match_token(Token::To);
        let end_value = self.evaluate_bitwise()?;
        self.match_token(Token::RightParantheses);

        self.match_token(Token::Begin);
        {
            let i = self.i;
            let mut control_var = *self.variables.get(&var).unwrap();
            while control_var <= end_value {
                self.evaluate_bitwise()?;

                if self.match_token(Token::End) {
                    if control_var + 1 > end_value {
                        break;
                    }
                    self.i = i;
                } else {
                    self.end_of_statement()?;
                }

                control_var += 1;
                self.variables.insert(var.to_string(), control_var);
            }

        }

        Ok(0)
    }

    fn end_of_statement(&mut self) -> Result<(), Error> {
        if self.match_token(Token::Semicolon) {
            return Ok(());
        }

        Err(Error::MissingSemicolon(self.current_token_info.clone()))
    }
}

pub fn parse(tokens: &[TokenInfo], variables: &mut HashMap<String, i64>) -> Result<i64, Error> {
    let mut parser_info = ParserInfo {
        tokens,
        current_token_info: TokenInfo {
            token: Token::None,
            lexeme: String::from(""),
            start_position: Position { row: 1, col: 1 },
        },
        i: 0,
        variables
    };

    let mut result = 0;
    while !parser_info.match_token(Token::EOF) {
        result += parser_info.evaluate_bitwise()?;
        if parser_info.match_token(Token::EOF) {
            break;
        } else {
            parser_info.end_of_statement()?;
        }
    }

    return Ok(result);
}
