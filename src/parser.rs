use std::{collections::HashSet, fmt, iter::Peekable};

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum LexerError {
    #[error("Unexpected character")]
    #[diagnostic(help("expected to find {}", expected))]
    UnexpectedCharacter {
        #[label("here")]
        at: SourceSpan,
        expected: &'static str,
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF {
        #[label("this was the last token")]
        last_token: Token,
    },
}

#[derive(Error, Diagnostic, Debug)]
pub enum ParserError {
    #[error("Unexpected token")]
    #[diagnostic(help("expected to find {}", expected))]
    UnexpectedToken {
        #[label("here")]
        at: SourceSpan,
        expected: &'static str,
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("Missing section")]
    #[diagnostic(help("expected to find {}", section))]
    MissingSection { section: &'static str },
    #[error("Duplicate section")]
    DuplicateSection {
        #[label("here")]
        at: SourceSpan,
        #[label("already defined here")]
        other: SourceSpan,
    },
    #[error("Unknown state")]
    UnknownState {
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Unknown alphabet symbol")]
    UnknownAlphabetSymbol {
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Unknown section name")]
    UnknownSectionName {
        #[label("here")]
        at: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    LeftSquareBracket,
    RightSquareBracket,
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Arrow,
    Identifier,
    Push,
    Pop,
    Noop,
    Write,
    Left,
    Right,
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

impl Token {
    pub fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub fn src(&self, src: &'static str) -> &'static str {
        &src[self.span.offset()..self.span.offset() + self.span.len()]
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::LeftSquareBracket => write!(f, "["),
            TokenKind::RightSquareBracket => write!(f, "]"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "=>"),
            TokenKind::Identifier => write!(f, "<identifier>"),
            TokenKind::Push => write!(f, "PUSH:<symbol>"),
            TokenKind::Pop => write!(f, "POP"),
            TokenKind::Noop => write!(f, "NOOP"),
            TokenKind::Write => write!(f, "WRITE:<symbol>"),
            TokenKind::Left => write!(f, "LEFT"),
            TokenKind::Right => write!(f, "RIGHT"),
            TokenKind::EOF => write!(f, "<EOF>"),
        }
    }
}

impl From<Token> for SourceSpan {
    fn from(token: Token) -> Self {
        token.span
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TransitionFrom {
    pub initial: Token,
    pub with_symbol: Token,
    pub with_stack_symbol: Option<Token>,
}

#[derive(Debug, Clone)]
pub enum StackTransition {
    Push((), Token), // PUSH:symbol
    Pop(()),
    NoOp(()),
    Write((), Token), // WRITE:symbol
}

#[derive(Debug, Clone)]
pub enum Direction {
    Left(()),
    Right(()),
}

#[derive(Debug, Clone)]
pub struct TransitionTo(
    pub Token,
    pub Option<StackTransition>,
    pub Option<Direction>,
);

#[derive(Debug)]
pub struct TransitionInfo {
    pub from: TransitionFrom,
    pub to: TransitionTo,
}

#[derive(Debug)]
pub struct PartialMachineInfo {
    pub states: Vec<Token>,
    pub alphabet: Vec<Token>,
    pub transitions: Vec<TransitionInfo>,
    pub start_state: Token,
    pub final_states: Vec<Token>,

    pub stack_alphabet: Option<Vec<Token>>,
    pub start_stack: Option<Token>,

    pub tape_alphabet: Option<Vec<Token>>,
    pub blank_symbol: Option<Token>,
}

pub struct Parser;

impl Parser {
    pub fn lex(input: &'static str) -> miette::Result<Vec<Token>> {
        let mut tokens = vec![];

        let eof_span = SourceSpan::new(input.len().into(), 0);
        let mut input = input.char_indices().peekable();

        while let Some((i, ch)) = input.next() {
            match ch {
                // One char tokens
                '[' => tokens.push(Token::new(
                    TokenKind::LeftSquareBracket,
                    SourceSpan::new(i.into(), 1),
                )),
                ']' => tokens.push(Token::new(
                    TokenKind::RightSquareBracket,
                    SourceSpan::new(i.into(), 1),
                )),
                '(' => tokens.push(Token::new(
                    TokenKind::LeftParen,
                    SourceSpan::new(i.into(), 1),
                )),
                ')' => tokens.push(Token::new(
                    TokenKind::RightParen,
                    SourceSpan::new(i.into(), 1),
                )),
                ',' => tokens.push(Token::new(TokenKind::Comma, SourceSpan::new(i.into(), 1))),
                ':' => tokens.push(Token::new(TokenKind::Colon, SourceSpan::new(i.into(), 1))),
                // Two char tokens
                '=' => {
                    if let Some((_, ch)) = input.peek() {
                        if *ch != '>' {
                            return Err(LexerError::UnexpectedCharacter {
                                at: SourceSpan::new(i.into(), 1),
                                expected: ">",
                            }
                            .into());
                        }
                        input.next();
                        tokens.push(Token::new(TokenKind::Arrow, SourceSpan::new(i.into(), 2)));
                    } else {
                        return Err(LexerError::UnexpectedEOF {
                            last_token: tokens.last().unwrap().clone(),
                        }
                        .into());
                    }
                }
                // Spaces
                ' ' | '\t' | '\n' | '\r' => {}
                // Comments
                '#' => {
                    while let Some((_, ch)) = input.peek() {
                        if *ch == '\n' {
                            break;
                        }

                        input.next();
                    }
                }
                // Identifiers
                ch => {
                    let mut identifier = ch.to_string();
                    while let Some((_, ch)) = input.peek() {
                        if ch.is_alphanumeric() || *ch == '_' {
                            identifier.push(*ch);
                            input.next();
                        } else {
                            break;
                        }
                    }

                    match identifier.as_str() {
                        "PUSH" => {
                            tokens.push(Token::new(
                                TokenKind::Push,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        "POP" => {
                            tokens.push(Token::new(
                                TokenKind::Pop,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        "NOOP" => {
                            tokens.push(Token::new(
                                TokenKind::Noop,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        "WRITE" => {
                            tokens.push(Token::new(
                                TokenKind::Write,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        "LEFT" => {
                            tokens.push(Token::new(
                                TokenKind::Left,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        "RIGHT" => {
                            tokens.push(Token::new(
                                TokenKind::Right,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                        _ => {
                            tokens.push(Token::new(
                                TokenKind::Identifier,
                                SourceSpan::new(i.into(), identifier.len()),
                            ));
                        }
                    }
                }
            }
        }

        tokens.push(Token::new(TokenKind::EOF, eof_span));
        Ok(tokens)
    }

    pub fn parse(src: &'static str, input: Vec<Token>) -> miette::Result<PartialMachineInfo> {
        let mut states = None;
        let mut alphabet = None;
        let mut stack_alphabet = None;
        let mut start_stack = None;
        let mut transitions = None;
        let mut start_state = None;
        let mut final_states = None;
        let mut tape_alphabet = None;
        let mut blank_symbol = None;

        let mut seen_sections: HashSet<Token> = HashSet::new();
        let ref mut input = input.into_iter().peekable();

        loop {
            let section = Self::parse_section(input)?;
            let section = match section {
                Some(section) => section,
                None => break,
            };

            if let Some(token) = seen_sections.get(&section) {
                return Err(ParserError::DuplicateSection {
                    at: section.span,
                    other: token.span,
                }
                .into());
            }

            match section.src(src) {
                "initial" => {
                    start_state = Some(Parser::parse_single_section(input)?);
                }
                "final" => {
                    final_states = Some(Parser::parse_list_section(input)?);
                }
                "states" => {
                    states = Some(Parser::parse_list_section(input)?);
                }
                "alphabet" => {
                    alphabet = Some(Parser::parse_list_section(input)?);
                }
                "stack_alphabet" => {
                    stack_alphabet = Some(Parser::parse_list_section(input)?);
                }
                "start_stack" => {
                    start_stack = Some(Parser::parse_single_section(input)?);
                }
                "tape_alphabet" => {
                    tape_alphabet = Some(Parser::parse_list_section(input)?);
                }
                "blank_symbol" => {
                    blank_symbol = Some(Parser::parse_single_section(input)?);
                }
                "transitions" => {
                    transitions = Some(Parser::parse_transitions(input)?);
                }
                _ => {
                    return Err(ParserError::UnknownSectionName { at: section.span }.into());
                }
            };

            seen_sections.insert(section);
        }

        Ok(PartialMachineInfo {
            states: states.ok_or(ParserError::MissingSection { section: "states" })?,
            alphabet: alphabet.ok_or(ParserError::MissingSection {
                section: "alphabet",
            })?,
            transitions: transitions.ok_or(ParserError::MissingSection {
                section: "transitions",
            })?,
            start_state: start_state.ok_or(ParserError::MissingSection {
                section: "start_state",
            })?,
            final_states: final_states.ok_or(ParserError::MissingSection {
                section: "final_states",
            })?,
            stack_alphabet,
            start_stack,
            tape_alphabet,
            blank_symbol,
        })
    }

    fn parse_section(
        input: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> miette::Result<Option<Token>> {
        // Assert [
        let token = match input.next() {
            Some(token) => token,
            None => {
                return Ok(None);
            }
        };

        match token.kind {
            TokenKind::LeftSquareBracket => {}
            TokenKind::EOF => {
                return Ok(None);
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    at: token.span,
                    expected: "[",
                }
                .into());
            }
        };

        // Parse the name
        let name = {
            let token = input.next().unwrap();
            match token.kind {
                TokenKind::Identifier => token,
                TokenKind::EOF => {
                    return Err(ParserError::UnexpectedEOF.into());
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        at: token.span,
                        expected: "<identifier>",
                    }
                    .into());
                }
            }
        };

        // Assert ]
        let token = input.next().unwrap();
        match token.kind {
            TokenKind::RightSquareBracket => {}
            TokenKind::EOF => {
                return Err(ParserError::UnexpectedEOF.into());
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    at: token.span,
                    expected: "]",
                }
                .into());
            }
        };

        Ok(Some(name))
    }

    fn parse_single_section(
        input: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> miette::Result<Token> {
        let token = input.next().unwrap();
        match token.kind {
            TokenKind::Identifier => Ok(token),
            TokenKind::EOF => {
                return Err(ParserError::UnexpectedEOF.into());
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    at: token.span,
                    expected: "<identifier>",
                }
                .into());
            }
        }
    }

    fn parse_list_section(
        input: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> miette::Result<Vec<Token>> {
        let mut final_states = vec![];

        while let Some(token) = input.next() {
            match token.kind {
                TokenKind::Identifier => {
                    final_states.push(token);

                    // Assert comma or [
                    let token = input.peek().unwrap();
                    match token.kind {
                        TokenKind::Comma => input.next(),
                        TokenKind::EOF => {
                            return Ok(final_states);
                        }
                        // Next section
                        TokenKind::LeftSquareBracket => {
                            return Ok(final_states);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: token.span,
                                expected: ",",
                            }
                            .into());
                        }
                    };
                }
                TokenKind::EOF => {
                    return Err(ParserError::UnexpectedEOF.into());
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        at: token.span,
                        expected: "[",
                    }
                    .into());
                }
            }
        }

        panic!("reached end of iter without consuming EOF");
    }

    fn parse_transitions(
        input: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> miette::Result<Vec<TransitionInfo>> {
        let mut transitions = vec![];

        while let Some(token) = input.next() {
            match token.kind {
                TokenKind::Identifier => {
                    // Assert left paren
                    let left_paren_token = input.next().unwrap();
                    match left_paren_token.kind {
                        TokenKind::LeftParen => {}
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: left_paren_token.span,
                                expected: "(",
                            }
                            .into());
                        }
                    };

                    let letter_token = input.next().unwrap();
                    let letter_token = match letter_token.kind {
                        TokenKind::Identifier => letter_token,
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: letter_token.span,
                                expected: "<identifier>",
                            }
                            .into());
                        }
                    };

                    // Potential stack symbol
                    let mut stack_letter_token = None;
                    if input.peek().unwrap().kind == TokenKind::Comma {
                        // Pass the comma
                        input.next().unwrap();
                        let stack_letter = input.next().unwrap();
                        match stack_letter.kind {
                            TokenKind::Identifier => {
                                stack_letter_token = Some(stack_letter);
                            }
                            TokenKind::EOF => {
                                return Err(ParserError::UnexpectedEOF.into());
                            }
                            _ => {
                                return Err(ParserError::UnexpectedToken {
                                    at: stack_letter.span,
                                    expected: "<identifier>",
                                }
                                .into());
                            }
                        }
                    }

                    // Assert right paren
                    let right_paren_token = input.next().unwrap();
                    match right_paren_token.kind {
                        TokenKind::RightParen => {}
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: right_paren_token.span,
                                expected: ")",
                            }
                            .into());
                        }
                    };

                    // Assert arrow
                    let arrow_token = input.next().unwrap();
                    match arrow_token.kind {
                        TokenKind::Arrow => {}
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: arrow_token.span,
                                expected: "=>",
                            }
                            .into());
                        }
                    };

                    // Assert left paren
                    let left_paren_token = input.next().unwrap();
                    match left_paren_token.kind {
                        TokenKind::LeftParen => {}
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: left_paren_token.span,
                                expected: "(",
                            }
                            .into());
                        }
                    };

                    let next_state = input.next().unwrap();
                    let next_state_token = match next_state.kind {
                        TokenKind::Identifier => next_state,
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        // Next section
                        TokenKind::LeftSquareBracket => {
                            return Ok(transitions);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: next_state.span,
                                expected: "<identifier>",
                            }
                            .into());
                        }
                    };

                    let mut stack_next_state_token = None;
                    let mut direction_token = None;

                    // First, check for stack operations (PDA syntax)
                    if input.peek().unwrap().kind == TokenKind::Comma {
                        // Pass the comma
                        input.next().unwrap();
                        let op_token = input.next().unwrap();
                        match op_token.kind {
                            TokenKind::Push => {
                                // Assert :
                                let colon_token = input.next().unwrap();
                                match colon_token.kind {
                                    TokenKind::Colon => {}
                                    TokenKind::EOF => {
                                        return Err(ParserError::UnexpectedEOF.into());
                                    }
                                    _ => {
                                        return Err(ParserError::UnexpectedToken {
                                            at: colon_token.span,
                                            expected: ":",
                                        }
                                        .into());
                                    }
                                };

                                // We need our identifier now
                                let stack_next_state_iden = input.next().unwrap();
                                match stack_next_state_iden.kind {
                                    TokenKind::Identifier => {
                                        stack_next_state_token = Some(StackTransition::Push(
                                            (), // Was op_token
                                            stack_next_state_iden,
                                        ));
                                    }
                                    TokenKind::EOF => {
                                        return Err(ParserError::UnexpectedEOF.into());
                                    }
                                    _ => {
                                        return Err(ParserError::UnexpectedToken {
                                            at: stack_next_state_iden.span,
                                            expected: "<identifier>",
                                        }
                                        .into());
                                    }
                                }
                            }
                            TokenKind::Pop => {
                                stack_next_state_token = Some(StackTransition::Pop(()));
                            }
                            TokenKind::Noop => {
                                stack_next_state_token = Some(StackTransition::NoOp(()));
                            }

                            // TM tape operations
                            TokenKind::Write => {
                                // Assert :
                                let colon_token = input.next().unwrap();
                                match colon_token.kind {
                                    TokenKind::Colon => {}
                                    TokenKind::EOF => {
                                        return Err(ParserError::UnexpectedEOF.into());
                                    }
                                    _ => {
                                        return Err(ParserError::UnexpectedToken {
                                            at: colon_token.span,
                                            expected: ":",
                                        }
                                        .into());
                                    }
                                };

                                // We need the symbol to write
                                let write_symbol = input.next().unwrap();
                                match write_symbol.kind {
                                    TokenKind::Identifier => {
                                        stack_next_state_token =
                                            Some(StackTransition::Write((), write_symbol));
                                    }
                                    TokenKind::EOF => {
                                        return Err(ParserError::UnexpectedEOF.into());
                                    }
                                    _ => {
                                        return Err(ParserError::UnexpectedToken {
                                            at: write_symbol.span,
                                            expected: "<identifier>",
                                        }
                                        .into());
                                    }
                                }
                            }
                            TokenKind::Left => {
                                direction_token = Some(Direction::Left(()));
                            }
                            TokenKind::Right => {
                                direction_token = Some(Direction::Right(()));
                            }
                            TokenKind::EOF => {
                                return Err(ParserError::UnexpectedEOF.into());
                            }
                            _ => {
                                return Err(ParserError::UnexpectedToken {
                                    at: op_token.span,
                                    expected: "PUSH, POP, NOOP, WRITE, LEFT, or RIGHT",
                                }
                                .into());
                            }
                        }

                        // Check for another comma for additional operations (e.g., both WRITE and direction)
                        if input.peek().unwrap().kind == TokenKind::Comma {
                            // Pass the comma
                            input.next().unwrap();
                            let dir_token = input.next().unwrap();
                            match dir_token.kind {
                                TokenKind::Left => {
                                    direction_token = Some(Direction::Left(()));
                                }
                                TokenKind::Right => {
                                    direction_token = Some(Direction::Right(()));
                                }
                                TokenKind::EOF => {
                                    return Err(ParserError::UnexpectedEOF.into());
                                }
                                _ => {
                                    return Err(ParserError::UnexpectedToken {
                                        at: dir_token.span,
                                        expected: "LEFT, RIGHT, or STAY",
                                    }
                                    .into());
                                }
                            }
                        }
                    }

                    // Assert right paren
                    let right_paren_token = input.next().unwrap();
                    match right_paren_token.kind {
                        TokenKind::RightParen => {}
                        TokenKind::EOF => {
                            return Err(ParserError::UnexpectedEOF.into());
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                at: right_paren_token.span,
                                expected: ")",
                            }
                            .into());
                        }
                    };

                    transitions.push(TransitionInfo {
                        from: TransitionFrom {
                            initial: token,
                            with_symbol: letter_token,
                            with_stack_symbol: stack_letter_token,
                        },
                        to: TransitionTo(next_state_token, stack_next_state_token, direction_token),
                    });
                }
                TokenKind::EOF => {
                    return Ok(transitions);
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        at: token.span,
                        expected: "[",
                    }
                    .into());
                }
            }
        }

        println!("{:?}", transitions);
        panic!("reached end of iter without consuming EOF");
    }
}
