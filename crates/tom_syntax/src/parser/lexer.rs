//! FIXME: write short doc here

use m_lexer::{Lexer, LexerBuilder, TokenKind};
use lazy_static::lazy_static;

use crate::{symbol, Symbol, TextRange, TextSize};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Token {
    pub symbol: Symbol,
    pub range: TextRange,
}

impl Token {
    pub fn is_significant(self) -> bool {
        !matches!(self.symbol, symbol::WHITESPACE | symbol::COMMENT)
    }
}

#[derive(Debug)]
pub(crate) struct Tokens {
    pub raw_tokens: Vec<Token>,
    pub significant: Vec<usize>,
}

pub(crate) fn tokenize(input: &str) -> Tokens {
    let raw_tokens = LEXER.tokenize(input)
        .iter()
        .enumerate()
        .map({
            let mut offset: TextSize = 0.into();
            |token| {
                let len: TextSize = token.len.try_into().unwrap();
                let token_offset = offset;
                offset += len;
                Token {
                    symbol: Symbol::new(token.kind.0),
                    range: TextRange::at(token_offset, len),
                }
            }
        })
        .collect();

    let significant = raw_tokens
        .iter()
        .enumerate()
        .filter(|(_idx, tok)| tok.is_significant())
        .map(|(idx, _tok)| idx)
        .collect();
    Tokens {
        raw_tokens,
        significant,
    }
}

lazy_static! {
    static ref LEXER: Lexer = lexer();
}

fn lexer() -> Lexer {
    fn t(s: Symbol) -> TokenKind {
        TokenKind(s.0.get() as u16)
    }

    LexerBuilder::new()
        .error_token(t(symbol::ERROR))
        .tokens(&[
            (t(symbol::EQ), r"="),
            (t(symbol::DOT), r"\."),
            (t(symbol::COMMA), r","),
            (t(symbol::L_BRACK), r"\["),
            (t(symbol::R_BRACK), r"\]"),
            (t(symbol::L_CURLY), r"\{"),
            (t(symbol::R_CURLY), r"\}"),
            (t(symbol::WHITESPACE), r"\s+"),
            (t(symbol::COMMENT), r"#.*"),
            (t(symbol::BARE_KEY_OR_NUMBER), r"[0-9]+"),
            (
                t(symbol::BARE_KEY_OR_DATE),
                r"[0-9]{4}-[0-9]{2}-[0-9]{2}[Zz]?",
            ),
            (
                t(symbol::BASIC_STRING),
                r#"(?x)
                    " ([^\r\n"\\] | \\.)* "
                "#,
            ),
            (
                t(symbol::MULTILINE_BASIC_STRING),
                r#"(?x)
                    """ ([^"] | \\. | "[^"] | ""[^"])* """
                "#,
            ),
            (
                t(symbol::LITERAL_STRING),
                r#"(?x)
                    ' ([^\r\n'] | \\.)* '
                "#,
            ),
            (
                t(symbol::MULTILINE_LITERAL_STRING),
                r#"(?x)
                    ''' ([^'] | \\. | '[^'] | ''[^'])* '''
                "#,
            ),
            (t(symbol::BOOL), r"(false|true)"),
            (
                t(symbol::NUMBER),
                r"(?x)
                    [-+]?
                    (0|[1-9](_?[0-9])*) # no leading zeros
                    (\.[0-9](_?[0-9])*)?
                    ([eE][-+]?[1-9](_?[0-9])*)?
                ",
            ),
            (
                t(symbol::DATE_TIME),
                r"(?x)
                    ( [0-9]{4}-[0-9]{2}-[0-9]{2} ([Tt]([0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]+)?))?
                    | ([0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]+)?))
                    ([Zz]|[+-][0-9]{2}:[0-9]{2})?
                ",
            ),
            (t(symbol::BARE_KEY), r"[0-9_\-a-zA-Z]+"),
        ])
        .build()
}
