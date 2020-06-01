//! This module defines Concrete Syntax Tree (CST), used by tom.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! module just wraps its API.

use std::hash::{Hash, Hasher};

use rowan::{Language, GreenNodeBuilder};
pub use rowan::{Direction, NodeOrToken, GreenNode};

use crate::{Symbol, SyntaxError, Parse, SmolStr};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TomlLanguage {}
impl Language for TomlLanguage {
    type Kind = Symbol;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Symbol {
        Symbol::new(raw.0)
    }
    fn kind_to_raw(kind: Symbol) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.0.get().into())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<TomlLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<TomlLanguage>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<TomlLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<TomlLanguage>;


// TODO: remove or implement?
// impl<R: TreeRoot<TomlLanguage>> fmt::Debug for SyntaxNode<R> {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//         write!(fmt, "{:?}@{:?}", self.symbol(), self.range())?;
//         // if has_short_text(self.kind()) {
//         //     write!(fmt, " \"{}\"", self.text())?;
//         // }
//         Ok(())
//     }
// }

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<SyntaxError>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<SyntaxError>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        Parse::new(green, errors);
    }

    pub fn token(&mut self, kind: Symbol, text: SmolStr) {
        let kind = TomlLanguage::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: Symbol) {
        let kind = TomlLanguage::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }
}
