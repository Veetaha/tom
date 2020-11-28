//! FIXME: write short doc here

uncover::define_uncover_macros!(enable_if(cfg!(debug_assertions)));

mod syntax_node;
mod syntax_error;
mod parser;
// mod model;
// mod visitor;
mod validator;
// mod edit;
mod syntax_kind;

pub mod ast;
// TODO: remove
// pub mod symbol;

use std::{marker::PhantomData, rc::Rc};

// pub use edit::{IntoValue, Position};
// pub use model::{Item, Map};

// TODO: remove TextUnit:
pub use rowan::{SmolStr, TextRange, TextSize, TextSize as TextUnit, WalkEvent, GreenNode};

pub use syntax_node::{SyntaxNode, SyntaxToken, SyntaxNodeChildren};
pub use syntax_kind::SyntaxKind;
pub use syntax_error::SyntaxError;
use ast::AstNode;
use std::fmt::Write;

/// `Parse` is the result of the parsing: a syntax tree and a collection of
/// errors.
///
/// Note that we always produce a syntax tree, even for completely invalid
/// files.
#[derive(Debug, PartialEq, Eq)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Rc<Vec<SyntaxError>>, // TODO: it was Arc<...>
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse {
            green: self.green.clone(),
            errors: self.errors.clone(),
            _ty: PhantomData,
        }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse {
            green,
            errors: Rc::new(errors),
            _ty: PhantomData,
        }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

impl<T: AstNode> Parse<T> {
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse {
            green: self.green,
            errors: self.errors,
            _ty: PhantomData,
        }
    }

    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &*self.errors
    }

    pub fn ok(self) -> Result<T, Rc<Vec<SyntaxError>>> {
        // TODO: it was Arc<...>
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

impl Parse<SyntaxNode> {
    pub fn cast<N: AstNode>(self) -> Option<Parse<N>> {
        if N::cast(self.syntax_node()).is_some() {
            Some(Parse {
                green: self.green,
                errors: self.errors,
                _ty: PhantomData,
            })
        } else {
            None
        }
    }
}

impl Parse<ast::Doc> {
    pub fn debug_dump(&self) -> String {
        let mut buf = format!("{:#?}", self.tree().syntax());
        for err in self.errors.iter() {
            writeln!(buf, "error {:?}: {}\n", err.range(), err).unwrap(); // TODO: replace with format_to!()
        }
        buf
    }

    // TODO: implement or remove:
    // pub fn reparse(&self, indel: &Indel) -> Parse<SourceFile> {
    //     self.incremental_reparse(indel).unwrap_or_else(|| self.full_reparse(indel))
    // }

    // fn incremental_reparse(&self, indel: &Indel) -> Option<Parse<SourceFile>> {
    //     // FIXME: validation errors are not handled here
    //     parsing::incremental_reparse(self.tree().syntax(), indel, self.errors.to_vec()).map(
    //         |(green_node, errors, _reparsed_range)| Parse {
    //             green: green_node,
    //             errors: Arc::new(errors),
    //             _ty: PhantomData,
    //         },
    //     )
    // }

    // fn full_reparse(&self, indel: &Indel) -> Parse<ast::Doc> {
    //     let mut text = self.tree().syntax().text().to_string();
    //     indel.apply(&mut text);
    //     SourceFile::parse(&text)
    // }
}
