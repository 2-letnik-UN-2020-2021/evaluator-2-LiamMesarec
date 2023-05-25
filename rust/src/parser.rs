use crate::tokenizer::{TokenInfo, Token, Position};

#[derive(Debug)]
pub enum Error {
    Generic(TokenInfo, String),
    InvalidFor(TokenInfo),
    InvalidAssignment(TokenInfo, String),
    MissingClosingBrackets(TokenInfo),
    MissingClosingParantheses(TokenInfo),
    ExpectedStartingBrackets(TokenInfo),
    ExpectedStartingParantheses(TokenInfo),
    MissingSemicolon(TokenInfo)
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
                write!(f, "Syntax error: missing semicolon ';' on line {}", token_info.start_position.row)

        }
    }
}

struct ParserInfo<'slice> {
    tokens:  &'slice [TokenInfo],
    current_token_info: TokenInfo,
    i: usize
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

    /*fn statement(&mut self, expected_tokens: &[Token]) -> Result<(), Error> {
        for &expected_token in expected_tokens {
            if !self.match_token(expected_token) {
                return Err(Error::Generic(self.current_token_info.clone(), self.last_n_token_lexemes(3)));
            }
        }

        return Ok(());
    }*/

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
}

pub fn parse(tokens: &[TokenInfo]) -> Result<(), Error> {
    let mut parser_info = ParserInfo {
        tokens,
        current_token_info: TokenInfo {
            token: Token::None,
            lexeme: String::from(""),
            start_position: Position { row: 1, col: 1 }
        },
        i: 0
    };

    while !parser_info.match_token(Token::EOF) {
        bitwise(&mut parser_info)?;
        if parser_info.match_token(Token::EOF) {
            break;
        } else {
            end_of_statement(&mut parser_info)?;
        }
    }

    Ok(())
}

fn bitwise(parser_info: &mut ParserInfo) -> Result<(), Error> {
    addition(parser_info)?;
    while parser_info.match_token(Token::BWAnd) || parser_info.match_token(Token::BWOr) {
        addition(parser_info)?;
    }

    Ok(())
}

fn addition(parser_info: &mut ParserInfo) -> Result<(), Error> {
    multiplication(parser_info)?;
    while parser_info.match_token(Token::Addition) || parser_info.match_token(Token::Subtraction) {
        multiplication(parser_info)?;
    }

    Ok(())
}

fn multiplication(parser_info: &mut ParserInfo) -> Result<(), Error> {
    comparison_operators(parser_info)?;
    while parser_info.match_token(Token::Multiplication) || parser_info.match_token(Token::Division) {
        comparison_operators(parser_info)?;
    }

    Ok(())
}

fn comparison_operators(parser_info: &mut ParserInfo) -> Result<(), Error> {
    unary(parser_info)?;
    while parser_info.match_token(Token::GreaterThan) || parser_info.match_token(Token::LowerThan) || parser_info.match_token(Token::Comparison) {
        unary(parser_info)?;
    }

    Ok(())
}

fn assignment(parser_info: &mut ParserInfo) -> Result<(), Error> {
    if parser_info.match_token(Token::Identifier) && parser_info.match_token(Token::Assignment) {
        return bitwise(parser_info);
    }

    Err(Error::InvalidAssignment(parser_info.current_token_info.clone(), parser_info.last_n_token_lexemes(3)))
}

fn end_of_statement(parser_info: &mut ParserInfo) -> Result<(), Error> {
    if parser_info.match_token(Token::Semicolon) {
        return Ok(());
    }

    Err(Error::MissingSemicolon(parser_info.current_token_info.clone()))
}

fn unary(parser_info: &mut ParserInfo) -> Result<(), Error> {
    if parser_info.match_token(Token::Addition) || parser_info.match_token(Token::Subtraction) {
        primary(parser_info)
    } else {
        primary(parser_info)
    }
}

fn primary(parser_info: &mut ParserInfo) -> Result<(), Error> {
    if parser_info.match_token(Token::Int) || parser_info.match_token(Token::Hex) {
        Ok(())
    } else if parser_info.match_token(Token::Identifier) {
        if parser_info.match_token(Token::Assignment) {
            bitwise(parser_info)
        } else {
            Ok(())
        }
    } else if parser_info.match_token(Token::LeftParantheses) {
        bitwise(parser_info)?;
        if !parser_info.match_token(Token::RightParantheses) {
            return Err(Error::MissingClosingParantheses(parser_info.current_token_info.clone()));
        }

        Ok(())
    } else if parser_info.match_token(Token::For) {
        if parser_info.match_token(Token::LeftParantheses) {
            assignment(parser_info)?;
            if !parser_info.match_token(Token::To) {
                return Err(Error::InvalidFor(parser_info.current_token_info.clone()));
            }

            bitwise(parser_info)?;

            if !parser_info.match_token(Token::RightParantheses) {
                return Err(Error::MissingClosingParantheses(parser_info.current_token_info.clone()));
            }

            if !parser_info.match_token(Token::Begin) {
                return Err(Error::MissingClosingParantheses(parser_info.current_token_info.clone()));
            }

            while !parser_info.match_token(Token::End) {
                bitwise(parser_info)?;

                if parser_info.match_token(Token::End) {
                    break;
                } else {
                    end_of_statement(parser_info)?;
                }
            }

            Ok(())
        } else {
            return Err(Error::ExpectedStartingParantheses(parser_info.current_token_info.clone()));
        }
    } else if parser_info.match_token(Token::While) {
        bitwise(parser_info)?;
        if !parser_info.match_token(Token::LeftBraces) {
                return Err(Error::ExpectedStartingBrackets(parser_info.current_token_info.clone()));
        }
        while !parser_info.match_token(Token::RightBraces) {
            if parser_info.match_token(Token::EOF) {
                return Err(Error::MissingClosingBrackets(parser_info.current_token_info.clone()));
            }
            bitwise(parser_info)?;
        }

        Ok(())
    } else if parser_info.match_token(Token::LeftBraces) {
        while !parser_info.match_token(Token::RightBraces) {
            if parser_info.match_token(Token::EOF) {
                return Err(Error::MissingClosingBrackets(parser_info.current_token_info.clone()));
            }
            bitwise(parser_info)?;
        }
        Ok(())
    } else if parser_info.match_token(Token::Console) {
        bitwise(parser_info)
    } else {
        return Err(Error::Generic(parser_info.current_token_info.clone(), parser_info.last_n_token_lexemes(3)));
    }

}
