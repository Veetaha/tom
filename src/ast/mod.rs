use ::{Children, TomlNode};
use std::marker::PhantomData;

mod generated;
pub use self::generated::*;
mod ext;
pub use self::ext::*;


pub trait AstNode<'f>: Copy {
    fn cast(node: TomlNode<'f>) -> Option<Self> where Self: Sized;

    fn node(self) -> TomlNode<'f>;
}

pub struct AstChildren<'f, A: AstNode<'f>> {
    inner: Children<'f>,
    phantom: PhantomData<*const A>,
}

impl<'f, A: AstNode<'f>> AstChildren<'f, A> {
    pub fn new(children: Children<'f>) -> Self {
        AstChildren {
            inner: children,
            phantom: PhantomData,
        }
    }
}

impl<'f, A: AstNode<'f>> Iterator for AstChildren<'f, A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.inner.next() {
            if let Some(a) = A::cast(node) {
                return Some(a);
            }
        }
        return None;
    }
}