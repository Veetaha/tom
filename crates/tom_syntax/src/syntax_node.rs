//! This module defines Concrete Syntax Tree (CST), used by tom.
//!
//! The CST includes comments and whitespace, provides a single node type,
//! `SyntaxNode`, and a basic traversal API (parent, children, siblings).
//!
//! The *real* implementation is in the (language-agnostic) `rowan` crate, this
//! module just wraps its API.

use std::hash::{Hash, Hasher};

use rowan::{Language, GreenNodeBuilder};
pub use rowan::{Direction, NodeOrToken};

use crate::{SyntaxError, Parse, SmolStr, SyntaxKind};

pub(crate) use rowan::GreenNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TomlLanguage {}
impl Language for TomlLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::new(raw.0)
    }
    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
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
    pub fn finish(self) -> Parse<SyntaxNode> {
        let green = self.inner.finish();
        Parse::new(green, self.errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: SmolStr) {
        self.inner.token(TomlLanguage::kind_to_raw(kind), text);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.inner.start_node(TomlLanguage::kind_to_raw(kind));
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }
}
