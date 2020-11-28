//! FIXME: write short doc here

mod generated;
mod extensions;

use std::{marker::PhantomData, borrow::Cow};

use crate::{SyntaxNode, ast, SyntaxNodeChildren, SmolStr, syntax_node::SyntaxToken, SyntaxKind};
pub use generated::{nodes::*, tokens::*};

/// The main trait to go from untyped `SyntaxNode` to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool;
    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &SmolStr {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    brand: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            brand: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

mod support {
    use super::{AstChildren, AstNode, SyntaxKind, SyntaxNode, SyntaxToken};

    pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().find_map(N::cast)
    }

    pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
        AstChildren::new(parent)
    }

    pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
        parent
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind() == kind)
    }
}

pub trait EntriesOwner: AstNode {
    fn entries(&self) -> AstChildren<ast::Entry> {
        support::children(self.syntax())
    }
}

pub trait TableHeaderOwner: AstNode {
    // TODO: was not Option<...>
    fn header(&self) -> Option<ast::TableHeader> {
        support::child(self.syntax())
    }
}

pub trait KeysOwner: AstNode {
    fn keys(&self) -> AstChildren<ast::Key> {
        support::children(self.syntax())
    }
}

// TODO:
// impl ast::Key {
//     pub fn name(&self) -> Cow<'_, str> {
//         match self.kind() {
//             ast::KeyKind::StringLit(lit) => lit.value(),
//             ast::KeyKind::BareKey(key) => Cow::from(key.text()),
//         }
//     }
// }

// impl ast::StringLiteral {
//     pub fn value(&self) -> Cow<'_, str> {
//         //FIXME: broken completely
//         let text = self.text();
//         let len = text.len();
//         Cow::from(&text[1..len - 1])
//     }
// }

// impl ast::Bool {
//     pub fn value(&self) -> bool {
//         self.text() == "true"
//     }
// }

// impl ast::IntNumber {
//     pub fn value(&self) -> i64 {
//         self.text().parse().unwrap()
//     }
// }

// impl ast::FloatNumber {
//     pub fn value(&self) -> f64 {
//         self.text().parse().unwrap()
//     }
// }

// impl ast::DateTime {
//     // chrono?
//     pub fn value(self) -> ::std::time::SystemTime {
//         unimplemented!()
//     }
// }

// impl ast::Value {
//     pub fn as_string(self) -> Option<Cow<'a, str>> {
//         match self.kind() {
//             ast::ValueKind::StringLit(l) => Some(l.value()),
//             _ => None,
//         }
//     }

//     pub fn as_bool(self) -> Option<bool> {
//         match self.kind() {
//             ast::ValueKind::Bool(l) => Some(l.value()),
//             _ => None,
//         }
//     }

//     pub fn as_i64(self) -> Option<i64> {
//         match self.kind() {
//             ast::ValueKind::Number(l) => Some(l.value()),
//             _ => None,
//         }
//     }
// }
