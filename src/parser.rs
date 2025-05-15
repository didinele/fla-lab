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
    Arrow,
    Identifier,
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
            TokenKind::Arrow => write!(f, "=>"),
            TokenKind::Identifier => write!(f, "<identifier>"),
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
}

#[derive(Debug, Clone)]
pub struct TransitionTo(pub Token);

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

                    tokens.push(Token::new(
                        TokenKind::Identifier,
                        SourceSpan::new(i.into(), identifier.len()),
                    ));
                }
            }
        }

        tokens.push(Token::new(TokenKind::EOF, eof_span));
        Ok(tokens)
    }

    pub fn parse(src: &'static str, input: Vec<Token>) -> miette::Result<PartialMachineInfo> {
        let mut states = None;
        let mut alphabet = None;
        let mut transitions = None;
        let mut start_state = None;
        let mut final_states = None;

        let mut seen_sections: HashSet<Token> = HashSet::new();
        let ref mut input = input.into_iter().peekable();

        while seen_sections.len() < 5 {
            let section = Self::parse_section(input)?;
            if let Some(token) = seen_sections.get(&section) {
                return Err(ParserError::DuplicateSection {
                    at: section.span,
                    other: token.span,
                }
                .into());
            }

            match section.src(src) {
                "initial" => {
                    start_state = Some(Parser::parse_initial_section(input)?);
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
        })
    }

    fn parse_section(input: &mut Peekable<impl Iterator<Item = Token>>) -> miette::Result<Token> {
        // Assert [
        let token = input.next().unwrap();
        match token.kind {
            TokenKind::LeftSquareBracket => {}
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

        Ok(name)
    }

    fn parse_initial_section(
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

                    transitions.push(TransitionInfo {
                        from: TransitionFrom {
                            initial: token,
                            with_symbol: letter_token,
                        },
                        to: TransitionTo(next_state_token),
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
