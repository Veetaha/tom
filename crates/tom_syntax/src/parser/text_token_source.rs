//! See `TextTokenSource` docs.

use crate::{
    SyntaxKind::{self, EOF},
    TextRange, TextSize,
    parser::lexer::Token,
};

/// Abstracts tokens retrieval from source code text.
pub(crate) struct TextTokenSource<'t> {
    text: &'t str,
    /// tokens for which `is_trivia() == false`
    significant_tokens: Vec<Token>,

    /// Current position
    curr: usize,
}

impl<'t> TextTokenSource<'t> {
    fn at(&self, idx: usize) -> SyntaxKind {
        self.significant_tokens
            .get(idx)
            .map_or(EOF, |token| token.kind)
    }

    fn current(&self) -> SyntaxKind {
        self.at(self.curr)
    }

    fn lookahead_nth(&self, n: usize) -> SyntaxKind {
        self.at(self.curr + n)
    }

    fn bump(&mut self) {
        self.curr = usize::min(self.curr + 1, self.significant_tokens.len())
    }

    fn is_keyword(&self, kw: &str) -> bool {
        self.significant_tokens
            .get(self.curr)
            .map(|(token, offset)| &self.text[TextRange::at(*offset, token.len)] == kw)
            .unwrap_or(false)
    }

    /// Generate input from tokens(expect comment and whitespace).
    pub fn new(text: &'t str, raw_tokens: &'t [Token]) -> TextTokenSource<'t> {
        let token_offset_pairs: Vec<_> = raw_tokens
            .iter()
            .filter_map({
                let mut len = 0.into();
                move |token| {
                    let pair = if token.kind.is_trivia() {
                        None
                    } else {
                        Some((*token, len))
                    };
                    len += token.len;
                    pair
                }
            })
            .collect();

        let first = Self::syntax_kind_at(0, &token_offset_pairs);
        TextTokenSource {
            text,
            significant_tokens: token_offset_pairs,
            curr: (first, 0),
        }
    }
}
