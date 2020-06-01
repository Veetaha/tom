//! FIXME: write short doc here

mod grammar;
mod lexer;

use crate::{symbol::*, SyntaxNode, Symbol, SmolStr, TextRange, SyntaxError, syntax_node::SyntaxTreeBuilder};

pub(crate) fn parse(input: &str) -> SyntaxNode {
    let tokens = lexer::tokenize(input);
    let mut sink = TextTreeSink::new(input, &tokens);
    {
        let mut parser = Parser {
            sink: &mut sink,
            tokens: &tokens,
            pos: 0,
        };
        parser.parse();
    }
    let green = sink.builder.finish();
    SyntaxNode::new(green, sink.errors)
}

struct Parser<'s, 't: 's> {
    sink: &'s mut TextTreeSink<'t>,
    tokens: &'t lexer::Tokens,
    pos: usize,
}

/// Bridges the parser with our specific syntax tree representation.
///
/// `TextTreeSink` also handles attachment of trivia (whitespace) to nodes.
struct TextTreeSink<'t> {
    text: &'t str,
    tokens: &'t lexer::Tokens,
    // TODO: add this to track current position (and to report errors with positions)
    // text_pos: TextSize,
    token_pos: usize,
    builder: SyntaxTreeBuilder,
    errors: Vec<SyntaxError>,
}

impl<'t> TextTreeSink<'t> {
    fn new(text: &'t str, tokens: &'t lexer::Tokens) -> Self {
        TextTreeSink {
            token_pos: 0,
            text,
            tokens,
            builder: SyntaxTreeBuilder::default(),
            errors: Vec::new(),
        }
    }

    fn start(&mut self, s: Symbol) {
        let ws = self.whitespace();
        let n = self.leading_ws(ws, s);
        for _ in 0..(ws.len() - n) {
            self.bump(None)
        }
        self.builder.start_internal(s);
    }

    fn finish(&mut self, s: Symbol) {
        let ws = self.whitespace();
        let n = self.trailing_ws(ws, s);
        for _ in 0..n {
            self.bump(None)
        }
        self.builder.finish_internal();
    }

    fn token(&mut self, pos: usize, s: Option<Symbol>) {
        while self.token_pos < pos {
            self.bump(None)
        }
        self.bump(s);
    }

    fn error(&mut self, message: impl Into<String>) {
        if self.tokens.raw_tokens.is_empty() {
            // TODO: set proper range
            self.errors.push(SyntaxError::new(message, TextRange::empty(0.into())));
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

        self.errors.push(SyntaxError::new(message, tok.range)))
    }

    fn leading_ws(&self, ws: &[lexer::Token], s: Symbol) -> usize {
        match s {
            DOC => ws.len(),
            ENTRY | TABLE => {
                let mut adj_comments = 0;
                for (i, token) in ws.iter().rev().enumerate() {
                    match token.symbol {
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

    fn trailing_ws(&self, ws: &[lexer::Token], s: Symbol) -> usize {
        match s {
            DOC => ws.len(),
            ENTRY => {
                let mut adj_comments = 0;
                for (i, token) in ws.iter().enumerate() {
                    match token.symbol {
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

    fn bump(&mut self, s: Option<Symbol>) {
        let t = self.tokens.raw_tokens[self.token_pos];
        let s = s.unwrap_or(t.symbol);
        let text: SmolStr = self.text[t.range].into();
        self.builder.leaf(s, text);
        self.token_pos += 1;
    }
}
