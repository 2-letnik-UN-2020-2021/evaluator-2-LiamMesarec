use std::io::BufRead;

#[derive(Debug)]
pub enum Error {
    NotAKeyword(Token),
    InvalidPattern(String, Position),
    InvalidStream
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotAKeyword(token) =>
                write!(f, "Tokenizer error: not a keyword {}", token),
            Error::InvalidPattern(lexeme, position) =>
                write!(f, "Tokenizer error: invalid pattern {} on line {}", lexeme, position.row),
            Error::InvalidStream =>
                write!(f, "Tokenizer error: invalid stream. Cannot read"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    None = 0,
    Multiplication,
    Division,
    Addition,
    Subtraction,
    BWAnd,
    BWOr,
    Int,
    Hex,
    LeftParantheses,
    RightParantheses,
    LeftBraces,
    RightBraces,
    Identifier,
    Assignment,
    GreaterThan,
    LowerThan,
    Comparison,
    Semicolon,
    For,
    While,
    In,
    Range,
    Begin,
    End,
    To,
    Console,
    Ignore,
    EOT,
    EOF,
    Error
}

const MAX_STATE: usize = 31;

impl From<u32> for Token {
    fn from(i: u32) -> Self {
        match i {
            0 => Token::None,
            1 => Token::Multiplication,
            2 => Token::Division,
            3 => Token::Addition,
            4 => Token::Subtraction,
            5 => Token::BWAnd,
            6 => Token::BWOr,
            7 => Token::Int,
            8 => Token::Hex,
            9 => Token::LeftParantheses,
            10 => Token::RightParantheses,
            11 => Token::LeftBraces,
            12 => Token::RightBraces,
            13 => Token::Identifier,
            14 => Token::Assignment,
            15 => Token::GreaterThan,
            16 => Token::LowerThan,
            17 => Token::Comparison,
            18 => Token::Semicolon,
            19 => Token::For,
            20 => Token::While,
            21 => Token::In,
            22 => Token::Range,
            23 => Token::Begin,
            24 => Token::End,
            25 => Token::To,
            26 => Token::Console,
            27 => Token::Ignore,
            28 => Token::EOT,
            29 => Token::EOF,
            30 => Token::Error,
            _ => Token::None
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::None => write!(f, "NONE"),
            Token::Multiplication => write!(f, "MULTIPLICATION"),
            Token::Division => write!(f, "DIVISION"),
            Token::Addition => write!(f, "ADDITION"),
            Token::Subtraction => write!(f, "SUBTRACTION"),
            Token::BWAnd => write!(f, "BW_AND"),
            Token::BWOr => write!(f, "BW_OR"),
            Token::Int => write!(f, "INT"),
            Token::Hex => write!(f, "HEX"),
            Token::LeftParantheses => write!(f, "LEFT_PARANTHESES"),
            Token::RightParantheses => write!(f, "RIGHT_PARANTHESES"),
            Token::LeftBraces => write!(f, "LEFT_BRACES"),
            Token::RightBraces => write!(f, "RIGHT_BRACES"),
            Token::Identifier => write!(f, "IDENTIFIER"),
            Token::Assignment => write!(f, "ASSIGNMENT"),
            Token::GreaterThan => write!(f, "GREATER_THAN"),
            Token::LowerThan => write!(f, "LOWER_THAN"),
            Token::Comparison => write!(f, "COMPARISON"),
            Token::Semicolon => write!(f, "SEMICOLON"),
            Token::For => write!(f, "FOR"),
            Token::While => write!(f, "WHILE"),
            Token::In => write!(f, "IN"),
            Token::Range => write!(f, "RANGE"),
            Token::Begin => write!(f, "BEGIN"),
            Token::End => write!(f, "END"),
            Token::To => write!(f, "TO"),
            Token::Console => write!(f, "CONSOLE"),
            Token::Ignore => write!(f, "IGNORE"),
            Token::EOT => write!(f, "EOT"),
            Token::EOF => write!(f, "EOF"),
            Token::Error => write!(f, "ERROR")
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub row: u32,
    pub col: u32
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub lexeme: String,
    pub start_position: Position
}

struct DFA {
    num_states: usize,
    alphabet: [char; 256],
    last: char,
    final_states: Vec<Token>,
    position: Position
}

pub fn tokenize<R: BufRead>(mut tokens_reader: R) -> Result<Vec<TokenInfo>, Error> {
    let mut dfa = DFA {
        num_states: MAX_STATE,
        alphabet: [char::default(); 256],
        last: char::default(),
        final_states: vec![Token::Int, Token::Hex, Token::End, Token::Multiplication,
            Token::Division, Token::Addition, Token::Subtraction, Token::EOF,
            Token::Identifier, Token::None, Token::LeftParantheses, Token::RightParantheses,
            Token::LeftBraces, Token::RightBraces, Token::Assignment, Token::Semicolon,
            Token::For, Token::While, Token::Begin, Token::To, Token::Console, Token::Ignore, Token::BWAnd, Token::BWOr, Token::Range, Token::In, Token::GreaterThan, Token::LowerThan, Token::Comparison],
        position: Position { row: 1, col: 1 }
    };

    let mut vec = Vec::new();

    for i in 0..=255 {
        dfa.alphabet[i] = char::from_u32(i as u32).unwrap();
    }

    match get_token(&mut tokens_reader, &mut dfa) {
        Ok(mut token_info) => {
            while token_info.token != Token::EOF {
                if token_info.token != Token::None {
                    vec.push(token_info);
                }

                token_info = match get_token(&mut tokens_reader, &mut dfa) {
                    Ok(token_info) => token_info,
                    Err(error) => return Err(error)
                }
            }
        },

        Err(error) => return Err(error)
    };

    vec.push(TokenInfo {
        token: Token::EOF,
        lexeme: String::from(""),
        start_position: dfa.position
    });

    Ok(vec)
}

fn get_token<R: BufRead>(mut tokens_reader: R, mut dfa: &mut DFA) -> Result<TokenInfo, Error>
{
    let transitions_table = create_transitions_table(dfa.alphabet.len(), dfa.num_states);
    let mut buffer = [0; 1];
    let mut token_info = TokenInfo {
        token: Token::None,
        lexeme: String::from(""),
        start_position: dfa.position
    };

    let mut state = Token::None;
    let mut code: char;

    if dfa.last != char::default() {
        code = dfa.last;
        dfa.last = char::default();
    }
    else {
        if tokens_reader.read(&mut buffer).unwrap() > 0 {
            code = buffer[0] as char;
            dfa.position = update_position(dfa.position, code);
        } else {
            token_info.token = Token::EOF;
            return Ok(token_info);
        }
    }

    //never again bruh # je koda za HEX STEVILO :sklduaaolsjdlasflasnd
    /*if code == '#' {
        while code != '\n' {
            if tokens_reader.read(&mut buffer).unwrap() > 0 {
                code = buffer[0] as char;
                dfa.position = update_position(dfa.position, code);
            } else {
                token_info.token = Token::EOF;
                return Ok(token_info);
            }
        }
    }*/

    loop {
        let next_state = transitions_table[state as usize][code as usize].into();
        if next_state == Token::EOT || next_state == Token::EOF {
            break;
        }

        if state == Token::None && next_state == Token::None && code != char::default() {
            token_info.lexeme.push(code);
            return Err(Error::InvalidPattern(token_info.lexeme, token_info.start_position));
        }

        if next_state == Token::None {
            break;
        }

        state = next_state;
        token_info.lexeme.push(code);

        if tokens_reader.read(&mut buffer).unwrap() > 0 {
            code = buffer[0] as char;
            dfa.last = code;
            dfa.position = update_position(dfa.position, code);
        } else {
            token_info.token = Token::EOF;
            return Ok(token_info);
        }
        
    }

    if dfa.final_states.contains(&state.into()) {
        token_info.token = state.into();
        token_info.token = assign_if_reserved_identifier(&token_info);
        if token_info.token == Token::None {
            Ok(token_info)
        } else {
            token_info.start_position.row = dfa.position.row;
            token_info.start_position.col = dfa.position.col;
            Ok(token_info)
        }
    } else {
        Err(Error::InvalidPattern(token_info.lexeme, token_info.start_position))
    }
}

fn assign_if_reserved_identifier(token_info: &TokenInfo) -> Token {
    match token_info.lexeme.as_ref() {
        "for" => Token::For,
        "while" => Token::While,
        "in" => Token::In,
        "begin" => Token::Begin,
        "end" => Token::End,
        "to" => Token::To,
        "CONSOLE" => Token::Console,
        _ => token_info.token
    }
}

fn create_transitions_table(alphabet_len: usize, num_states: usize) -> Vec<Vec<u32>> {
    let mut transitions_table: Vec<Vec<u32>> = vec![vec![Token::None as u32; alphabet_len]; num_states];

    let mut set_transition = |from: Token, c: char, to: Token| {
        transitions_table[from as usize][c as usize] = to as u32;
    };

    set_transition(Token::None, ';', Token::Semicolon);
    set_transition(Token::None, ':', Token::Assignment);
    set_transition(Token::Assignment, '=', Token::Assignment);

    for i in '0'..='9' {
        set_transition(Token::None, i, Token::Int);
        set_transition(Token::Int, i, Token::Int);
        set_transition(Token::Hex, i, Token::Hex);
        set_transition(Token::Identifier, i, Token::Identifier);
    }

    set_transition(Token::None, '+', Token::Addition);
    set_transition(Token::None, '-', Token::Subtraction);
    set_transition(Token::None, '*', Token::Multiplication);
    set_transition(Token::None, '/', Token::Division);
    set_transition(Token::None, '&', Token::BWAnd);
    set_transition(Token::None, '|', Token::BWOr);

    set_transition(Token::None, '>', Token::GreaterThan);
    set_transition(Token::None, '<', Token::LowerThan);

    set_transition(Token::None, '=', Token::Comparison);
    set_transition(Token::Comparison, '=', Token::Comparison);

    set_transition(Token::None, '#', Token::Hex);
    for i in 'A'..='F' {
        set_transition(Token::Hex, i, Token::Hex);
    }

    for i in 'a'..='f' {
        set_transition(Token::Hex, i, Token::Hex);
    }

    for i in 'a'..='z' {
        set_transition(Token::None, i, Token::Identifier);
        set_transition(Token::Identifier, i, Token::Identifier);
    }

    for i in 'A'..='Z' {
        set_transition(Token::None, i, Token::Identifier);
        set_transition(Token::Identifier, i, Token::Identifier);
    }

    set_transition(Token::None, ' ', Token::EOT);
    set_transition(Token::None, '\t', Token::EOT);
    set_transition(Token::None, '\n', Token::EOT);
    set_transition(Token::EOT, ' ', Token::EOT);
    set_transition(Token::EOT, '\t', Token::EOT);
    set_transition(Token::EOT, '\n', Token::EOT);

    set_transition(Token::None, '(', Token::LeftParantheses);
    set_transition(Token::None, ')', Token::RightParantheses);
    set_transition(Token::None, '{', Token::LeftBraces);
    set_transition(Token::None, '}', Token::RightBraces);

    set_transition(Token::None, '.', Token::Range);
    set_transition(Token::Range, '.', Token::Range);

    set_transition(Token::None, Token::EOF as u8 as char, Token::EOF);
    transitions_table
}

fn update_position(position: Position, code: char) -> Position {
    let mut pos = position;
    if code == '\n' {
        pos.row += 1;
        pos.col = 1;
        return pos;
    }

    pos.col += 1;

    pos
}
