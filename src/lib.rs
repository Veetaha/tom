use std::ptr;
use std::fmt;

use parse_tree::{ParseTree, NodeId};
use ast::AstNode;
//use edit::TreeEdit;

mod text;
mod symbol;
mod parse_tree;
mod parser;
mod edit;

pub use text::{TextUnit, TextRange};
pub use symbol::*;
pub mod ast;
pub use edit::Edit;

#[derive(Debug)]
pub struct TomlFile {
    parse_tree: ParseTree,
    text: String,
}

impl TomlFile {
    pub fn new(text: String) -> TomlFile {
        let parse_tree = parser::parse(&text);
        TomlFile { parse_tree, text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn parse_tree(&self) -> TomlNode {
        TomlNode {
            file: self,
            id: self.parse_tree.root(),
        }
    }

    pub fn ast(&self) -> ast::File {
        ast::File::cast(self.parse_tree()).unwrap()
    }

    pub fn edit(&self) -> Edit {
        Edit::new(self)
    }

    pub fn debug_dump(&self) -> String {
        let mut result = String::new();
        go(self.parse_tree(), &mut result, 0);
        return result;

        fn go(node: TomlNode, buff: &mut String, level: usize) {
            buff.push_str(&String::from("  ").repeat(level));
            buff.push_str(&format!("{:?}", node));

            if node.children().next().is_none() {
                let node_text = node.text();
                if !node_text.chars().all(char::is_whitespace) {
                    buff.push_str(&format!(" {:?}", node_text));
                }
            }
            buff.push('\n');
            for child in node.children() {
                go(child, buff, level + 1)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct TomlNode<'f> {
    file: &'f TomlFile,
    id: NodeId,
}

impl<'f> PartialEq<TomlNode<'f>> for TomlNode<'f> {
    fn eq(&self, other: &TomlNode) -> bool {
        self.id == other.id && ptr::eq(self.file, other.file)
    }
}

impl<'f> Eq for TomlNode<'f> {}

impl<'t> fmt::Debug for TomlNode<'t> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}@{:?}", self.node().symbol().name(), self.node().range())
    }
}

impl<'f> TomlNode<'f> {
    pub fn symbol(&self) -> Symbol {
        self.node().symbol()
    }

    pub fn range(&self) -> TextRange {
        self.node().range()
    }

    pub fn text(&self) -> &'f str {
        &self.file.text[self.range()]
    }

    pub fn parent(&self) -> Option<TomlNode<'f>> {
        self.node().parent().map(|id| TomlNode {
            file: self.file,
            id,
        })
    }

    pub fn children(&self) -> Children<'f> {
        Children {
            file: self.file,
            id: self.node().first_child(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.node().first_child().is_none()
    }

    fn node(&self) -> &parse_tree::Node {
        &self.file.parse_tree[self.id]
    }
}

pub struct Children<'f> {
    file: &'f TomlFile,
    id: Option<NodeId>,
}

impl<'f> Iterator for Children<'f> {
    type Item = TomlNode<'f>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.id.map(|id| {
            self.id = self.file.parse_tree[id].next_sibling();
            TomlNode { file: &self.file, id }
        })
    }
}