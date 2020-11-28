use crate::{AstNode, ast, SyntaxToken, SyntaxKind::*};

pub enum AtomLiteralKind {
    Bool(bool),
    DateTime,
    Float,
    Int,
    String { kind: StringKind, multiline: bool },
}

enum StringKind {
    Literal,
    Basic,
}

impl ast::AtomLiteral {
    pub fn token(&self) -> SyntaxToken {
        self.syntax()
            .children_with_tokens()
            .find(|it| !it.kind().is_trivia())
            .and_then(|it| it.into_token())
            .expect("Atom literal always has at least one atom value token which it wraps")
    }

    pub fn kind(&self) -> AtomLiteralKind {
        match self.token().kind() {
            DATE_TIME => AtomLiteralKind::DateTime,
            TRUE => AtomLiteralKind::Bool(true),
            FALSE => AtomLiteralKind::Bool(false),
            INT => AtomLiteralKind::Int,
            FLOAT => AtomLiteralKind::Float,
            LITERAL_LINE_STRING => AtomLiteralKind::String {
                kind: StringKind::Literal,
                multiline: false,
            },
            LITERAL_MULTILINE_STRING => AtomLiteralKind::String {
                kind: StringKind::Literal,
                multiline: false,
            },
            BASIC_LINE_STRING => AtomLiteralKind::String {
                kind: StringKind::Basic,
                multiline: false,
            },
            BASIC_MULTILINE_STRING => AtomLiteralKind::String {
                kind: StringKind::Basic,
                multiline: false,
            },
            it => unreachable!("{:?}", it),
        }
    }
}
