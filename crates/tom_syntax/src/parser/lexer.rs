//! FIXME: write short doc here

use std::convert::TryFrom;
use crate::{SyntaxKind, TextRange, TextSize, SyntaxError};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Token {
    pub kind: SyntaxKind,
    // TODO: replace with TextSize?
    pub range: TextRange,
}

pub(crate) fn tokenize(input: &str) -> (Vec<Token>, Vec<SyntaxError>) {
    let mut tokens = vec![];
    // FIXME: implement collecting errors from `toml_lexer` into this vector (use rust-analyzer impl as cheatsheet)
    let mut errors = vec![];
    let mut tokenizer = toml_lexer::Tokenizer::new(input).peekable();

    while let Some(toml_token) = tokenizer.next() {
        let kind = match toml_token.kind {
            toml_lexer::TokenKind::Unknown => SyntaxKind::ERROR,
            toml_lexer::TokenKind::Whitespace => SyntaxKind::WHITESPACE,
            toml_lexer::TokenKind::Newline => SyntaxKind::NEWLINE,
            toml_lexer::TokenKind::Comment => SyntaxKind::COMMENT,
            toml_lexer::TokenKind::Equals => SyntaxKind::EQ,
            toml_lexer::TokenKind::Period => SyntaxKind::DOT,
            toml_lexer::TokenKind::Comma => SyntaxKind::COMMA,
            // Appears in dates
            toml_lexer::TokenKind::Colon => SyntaxKind::COLON,
            toml_lexer::TokenKind::Plus => SyntaxKind::PLUS,
            toml_lexer::TokenKind::LeftBrace => SyntaxKind::L_CURLY,
            toml_lexer::TokenKind::RightBrace => SyntaxKind::R_CURLY,
            toml_lexer::TokenKind::LeftBracket => SyntaxKind::L_BRACK,
            toml_lexer::TokenKind::RightBracket => SyntaxKind::R_BRACK,
            toml_lexer::TokenKind::Keylike => SyntaxKind::BARE_KEY_LIKE,
            toml_lexer::TokenKind::StrLitSubtoken(it) => {
                let quotes = match it {
                    toml_lexer::StrLitSubtoken::LeadingQuotes(it) => it,
                    _ => unreachable!(
                        "String literals always have leading quotes as their first token"
                    ),
                };

                let kind = match quotes.kind {
                    toml_lexer::StrLitKind::Literal => match quotes.len {
                        toml_lexer::QuotesLen::X1 => SyntaxKind::LITERAL_LINE_STRING,
                        toml_lexer::QuotesLen::X3 => SyntaxKind::LITERAL_MULTILINE_STRING,
                    },
                    toml_lexer::StrLitKind::Basic => match quotes.len {
                        toml_lexer::QuotesLen::X1 => SyntaxKind::BASIC_LINE_STRING,
                        toml_lexer::QuotesLen::X3 => SyntaxKind::BASIC_MULTILINE_STRING,
                    },
                };

                let end = loop {
                    match tokenizer.peek() {
                        Some(toml_lexer::Token {
                            span,
                            kind: toml_lexer::TokenKind::StrLitSubtoken(toml_subtoken),
                        }) => {
                            tokenizer.next();
                            if matches!(toml_subtoken, toml_lexer::StrLitSubtoken::TrailingQuotes) {
                                break span.end;
                            }
                        }
                        Some(toml_token) => break toml_token.span.start,
                        None => {
                            tokenizer.next();
                            break input.len();
                        }
                    }
                };

                tokens.push(Token {
                    kind,
                    range: TextRange::new(
                        TextSize::try_from(toml_token.span.start).unwrap(),
                        TextSize::try_from(end).unwrap(),
                    ),
                });
                continue;
            }
        };

        tokens.push(Token {
            kind,
            range: TextRange::new(
                TextSize::try_from(toml_token.span.start).unwrap(),
                TextSize::try_from(toml_token.span.end).unwrap(),
            ),
        });
    }

    (tokens, errors)
}
