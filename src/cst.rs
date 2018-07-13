use std::cmp;

use {
    TomlDoc, Symbol, TextUnit, TextRange, ChunkedText,
    tree::{NodeId, TreeData},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CstNode(pub(crate) NodeId);

pub enum CstNodeKind<'a> {
    Leaf(&'a str),
    Internal(CstChildren<'a>),
}

impl CstNode {
    pub fn symbol(self, doc: &TomlDoc) -> Symbol {
        *match self.0.data(&doc.tree) {
            TreeData::Internal(s) => s,
            TreeData::Leaf((s, _)) => s,
        }
    }

    pub fn range(self, doc: &TomlDoc) -> TextRange {
        assert!(
            !doc.edit_in_progress,
            "range info is unavailable during edit"
        );
        doc.data[self.0.to_idx()].range
    }

    pub fn kind(self, doc: &TomlDoc) -> CstNodeKind {
        match self.0.data(&doc.tree) {
            TreeData::Leaf((_, idx)) => CstNodeKind::Leaf(doc.intern.resolve(*idx)),
            TreeData::Internal(_) => CstNodeKind::Internal(self.children(doc)),
        }
    }

    pub fn is_leaf(self, doc: &TomlDoc) -> bool {
        match self.kind(doc) {
            CstNodeKind::Leaf(_) => true,
            CstNodeKind::Internal(_) => false,
        }
    }

    pub fn parent(self, doc: &TomlDoc) -> Option<CstNode> {
        self.0.parent(&doc.tree).map(CstNode)
    }

    pub fn children(self, doc: &TomlDoc) -> CstChildren {
        CstChildren { doc, node: self }
    }

    pub fn next_sibling(self, doc: &TomlDoc) -> Option<CstNode> {
        self.0.next_sibling(&doc.tree).map(CstNode)
    }

    pub fn prev_sibling(self, doc: &TomlDoc) -> Option<CstNode> {
        self.0.prev_sibling(&doc.tree).map(CstNode)
    }

    pub fn get_text(self, doc: &TomlDoc) -> String {
        self.chunked_text(doc).to_string()
    }

    pub(crate) fn chunked_text<'a>(self, doc: &'a TomlDoc) -> impl ChunkedText + 'a {
        struct Chunks<'a> {
            root: CstNode,
            doc: &'a TomlDoc,
        }

        impl<'a> Chunks<'a> {
            fn go<F: FnMut(&str) -> Result<(), T>, T>(
                &self,
                node: CstNode,
                f: &mut F,
            ) -> Result<(), T> {
                match node.kind(self.doc) {
                    CstNodeKind::Leaf(text) => f(text)?,
                    CstNodeKind::Internal(children) => {
                        for child in children {
                            self.go(child, f)?;
                        }
                    }
                }
                Ok(())
            }
        }

        impl<'a> ChunkedText for Chunks<'a> {
            fn for_each_chunk<F: FnMut(&str) -> Result<(), T>, T>(
                &self,
                mut f: F,
            ) -> Result<(), T> {
                self.go(self.root, &mut f)
            }
        }

        Chunks { root: self, doc }
    }

    pub(crate) fn chunked_substring<'a>(
        self,
        doc: &'a TomlDoc,
        range: TextRange,
    ) -> impl ChunkedText + 'a {
        assert!(
            !doc.edit_in_progress,
            "range info is unavailable during edit"
        );

        struct Chunks<'a> {
            root: CstNode,
            doc: &'a TomlDoc,
            range: TextRange,
        }

        impl<'a> Chunks<'a> {
            fn go<F: FnMut(&str) -> Result<(), T>, T>(
                &self,
                node: CstNode,
                f: &mut F,
            ) -> Result<(), T> {
                let node_range = node.range(self.doc);
                let rel_range = match intersect(self.range, node_range) {
                    None => return Ok(()),
                    Some(range) => relative_range(node_range.start(), range),
                };
                match node.kind(self.doc) {
                    CstNodeKind::Leaf(text) => f(&text[rel_range])?,
                    CstNodeKind::Internal(children) => {
                        for child in children {
                            self.go(child, f)?;
                        }
                    }
                }
                Ok(())
            }
        }

        impl<'a> ChunkedText for Chunks<'a> {
            fn for_each_chunk<F: FnMut(&str) -> Result<(), T>, T>(
                &self,
                mut f: F,
            ) -> Result<(), T> {
                self.go(self.root, &mut f)
            }
        }
        Chunks {
            root: self,
            doc,
            range,
        }
    }

    pub fn debug(self, doc: &TomlDoc) -> String {
        if doc.edit_in_progress {
            format!("{}@[??:??)", self.symbol(doc).name())
        } else {
            format!("{}@{:?}", self.symbol(doc).name(), self.range(doc))
        }
    }
}

#[derive(Clone, Copy)]
pub struct CstChildren<'a> {
    doc: &'a TomlDoc,
    node: CstNode,
}

impl<'a> CstChildren<'a> {
    pub fn first(self) -> Option<CstNode> {
        self.node.0.first_child(&self.doc.tree).map(CstNode)
    }
    pub fn last(self) -> Option<CstNode> {
        self.node.0.last_child(&self.doc.tree).map(CstNode)
    }
    pub fn iter(self) -> CstChildrenIter<'a> {
        CstChildrenIter {
            doc: self.doc,
            curr: self.first(),
        }
    }
    pub fn rev(self) -> RevCstChildrenIter<'a> {
        RevCstChildrenIter {
            doc: self.doc,
            curr: self.last(),
        }
    }
}

impl<'a> IntoIterator for CstChildren<'a> {
    type Item = CstNode;
    type IntoIter = CstChildrenIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone)]
pub struct CstChildrenIter<'a> {
    pub(crate) doc: &'a TomlDoc,
    curr: Option<CstNode>,
}

impl<'a> Iterator for CstChildrenIter<'a> {
    type Item = CstNode;
    fn next(&mut self) -> Option<CstNode> {
        self.curr.map(|node| {
            self.curr = node.next_sibling(self.doc);
            node
        })
    }
}

#[derive(Clone)]
pub struct RevCstChildrenIter<'a> {
    doc: &'a TomlDoc,
    curr: Option<CstNode>,
}

impl<'a> Iterator for RevCstChildrenIter<'a> {
    type Item = CstNode;
    fn next(&mut self) -> Option<CstNode> {
        self.curr.map(|node| {
            self.curr = node.prev_sibling(self.doc);
            node
        })
    }
}

fn intersect(r1: TextRange, r2: TextRange) -> Option<TextRange> {
    let start = cmp::max(r1.start(), r2.start());
    let end = cmp::min(r1.end(), r2.end());
    if end > start {
        Some(TextRange::from_to(start, end))
    } else {
        None
    }
}

fn relative_range(offset: TextUnit, range: TextRange) -> TextRange {
    TextRange::from_to(range.start() - offset, range.end() - offset)
}
