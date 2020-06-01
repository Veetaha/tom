//! FIXME: write short doc here

uncover::define_uncover_macros!(enable_if(cfg!(debug_assertions)));

mod syntax_node;
mod syntax_error;
mod parser;
// mod model;
// mod visitor;
mod validator;
// mod edit;

pub mod ast;
pub mod symbol;

use std::{num::NonZeroU8, marker::PhantomData};

// pub use edit::{IntoValue, Position};
// pub use model::{Item, Map};

// TODO: remove TextUnit:
pub use rowan::{SmolStr, TextRange, TextSize, TextSize as TextUnit, WalkEvent};

pub use syntax_node::{SyntaxNode, SyntaxNodeChildren};
pub use syntax_error::SyntaxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(NonZeroU8);

#[derive(Clone)]
pub struct Parse {
    // TODO: change to `ast::Doc` or `ast::SourceFile` (it doesn't exist yet)
    root: SyntaxNode,
    validation_errors: Vec<SyntaxError>,
}

impl Parse {
    pub fn new(text: &str) -> Parse {
        let root = parser::parse(text);
        let mut doc = Parse {
            root,
            validation_errors: Vec::new(),
        };

        let validation_errors = validator::validate(&doc);
        doc.validation_errors = validation_errors;

        doc
    }

    pub fn cst(&self) -> SyntaxNodeRef {
        self.root.borrowed()
    }

    pub fn ast(&self) -> ast::Doc {
        ast::Doc::cast(self.cst()).unwrap()
    }
    // pub(crate) fn replace_with(&self, replacement: GreenNode) -> GreenNode {
    //     self.0.replace_with(replacement)
    // }

    // pub fn model(&self) -> Map {
    //     model::from_doc(self)
    // }

    pub fn errors(&self) -> Vec<SyntaxError> {
        self.root
            .root_data()
            .iter()
            .chain(self.validation_errors.iter())
            .cloned()
            .collect()
    }

    pub fn debug(&self) -> String {
        let mut buff = String::new();
        let mut level = 0;
        for event in self.cst().preorder() {
            match event {
                WalkEvent::Enter(node) => {
                    buff.push_str(&String::from("  ").repeat(level));
                    let range = node.range();
                    let symbol = node.symbol();
                    buff.push_str(&format!("{}@{:?}", symbol, range));
                    if let Some(text) = node.leaf_text() {
                        if !text.chars().all(char::is_whitespace) {
                            buff.push_str(&format!(" {:?}", text));
                        }
                    }
                    buff.push('\n');
                    level += 1;
                }
                WalkEvent::Leave(_) => {
                    level -= 1;
                }
            }
        }

        let errors = self.errors();
        if !errors.is_empty() {
            let text = self.cst().get_text();
            buff += "\n";
            for e in errors {
                let text = &text[e.range];
                buff += &format!("error@{:?} {:?}: {}\n", e.range(), text, e.message());
            }
        }
        buff
    }
}
