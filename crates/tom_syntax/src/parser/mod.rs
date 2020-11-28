//! FIXME: write short doc here

mod grammar;
mod lexer;
mod text_token_source;

use crate::{
    SyntaxNode,
    SyntaxKind::{self, *},
    SmolStr, TextRange, SyntaxError,
    syntax_node::SyntaxTreeBuilder,
    Parse,
};
use lexer::Token;

pub(crate) fn parse(input: &str) -> Parse<SyntaxNode> {
    let (tokens, _errors) = lexer::tokenize(input);
    let mut sink = TextTreeSink::new(input, &tokens);
    Parser {
        sink: &mut sink,
        tokens: &tokens,
        pos: 0,
    }
    .parse();
    sink.builder.finish()
}

struct Parser<'s, 't: 's> {
    sink: &'s mut TextTreeSink<'t>,
    tokens: &'t [Token],
    pos: usize,
}

/// Bridges the parser with our specific syntax tree representation.
///
/// `TextTreeSink` also handles attachment of trivia (whitespace) to nodes.
struct TextTreeSink<'t> {
    text: &'t str,
    tokens: &'t [Token],
    // TODO: add this to track current position (and to report errors with positions)
    // text_pos: TextSize,
    token_pos: usize,
    builder: SyntaxTreeBuilder,
    errors: Vec<SyntaxError>,
}

impl<'t> TextTreeSink<'t> {
    fn new(text: &'t str, tokens: &'t [Token]) -> Self {
        TextTreeSink {
            token_pos: 0,
            text,
            tokens,
            builder: SyntaxTreeBuilder::default(),
            errors: Vec::new(),
        }
    }

    fn start(&mut self, s: SyntaxKind) {
        let ws = self.whitespace();
        let n = self.leading_ws(ws, s);
        for _ in 0..(ws.len() - n) {
            self.bump(None)
        }
        self.builder.start_node(s);
    }

    fn finish(&mut self, s: SyntaxKind) {
        let ws = self.whitespace();
        let n = self.trailing_ws(ws, s);
        for _ in 0..n {
            self.bump(None)
        }
        self.builder.finish_node()
    }

    fn token(&mut self, pos: usize, s: Option<SyntaxKind>) {
        while self.token_pos < pos {
            self.bump(None)
        }
        self.bump(s);
    }

    fn error(&mut self, message: impl Into<String>) {
        if self.tokens.raw_tokens.is_empty() {
            // TODO: set proper range
            self.errors
                .push(SyntaxError::new(message, TextRange::empty(0.into())));
            return;
        }

        let mut pos = self.token_pos;
        if pos == self.tokens.raw_tokens.len() {
            pos -= 1;
        }
        let mut tok = &self.tokens.raw_tokens[pos];
        loop {
            match &self.tokens.raw_tokens.get(pos) {
                Some(t) if t.is_significant() => {
                    tok = t;
                    break;
                }
                Some(t) => {
                    tok = t;
                }
                None => break,
            }
            pos += 1;
        }

        self.errors.push(SyntaxError::new(message, tok.range))
    }

    fn leading_ws(&self, ws: &[lexer::Token], s: SyntaxKind) -> usize {
        match s {
            DOC => ws.len(),
            ENTRY | TABLE => {
                let mut adj_comments = 0;
                for (i, token) in ws.iter().rev().enumerate() {
                    match token.kind {
                        COMMENT => {
                            adj_comments = i + 1;
                        }
                        WHITESPACE => {
                            let text = &self.text[token.range];
                            if text.bytes().filter(|&b| b == b'\n').count() >= 2 {
                                break;
                            }
                        }
                        c => unreachable!("not a ws: {:?}", c),
                    }
                }
                adj_comments
            }
            _ => 0,
        }
    }

    fn trailing_ws(&self, ws: &[lexer::Token], s: SyntaxKind) -> usize {
        match s {
            DOC => ws.len(),
            ENTRY => {
                let mut adj_comments = 0;
                for (i, token) in ws.iter().enumerate() {
                    match token.kind {
                        COMMENT => {
                            adj_comments = i + 1;
                        }
                        WHITESPACE => {
                            let text = &self.text[token.range];
                            if text.contains('\n') {
                                break;
                            }
                        }
                        c => unreachable!("not a ws: {:?}", c),
                    }
                }
                adj_comments
            }
            _ => 0,
        }
    }

    fn whitespace(&self) -> &'t [lexer::Token] {
        let start = self.token_pos;
        let mut end = start;
        loop {
            match &self.tokens.raw_tokens.get(end) {
                Some(token) if !token.is_significant() => end += 1,
                _ => break,
            }
        }
        &self.tokens.raw_tokens[start..end]
    }

    fn bump(&mut self, s: Option<SyntaxKind>) {
        let t = self.tokens.raw_tokens[self.token_pos];
        let s = s.unwrap_or(t.symbol);
        let text: SmolStr = self.text[t.range].into();
        self.builder.leaf(s, text);
        self.token_pos += 1;
    }
}
