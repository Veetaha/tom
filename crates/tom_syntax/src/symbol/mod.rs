//! FIXME: write short doc here

mod generated;

use std::{fmt, num::NonZeroU8, convert::TryInto};

use crate::Symbol;

pub use self::generated::*;

pub(crate) struct SymbolInfo(pub &'static str);

impl Symbol {
    pub(crate) fn new(idx: u16) -> Symbol {
        let idx: u8 = idx.try_into().unwrap();
        Symbol(NonZeroU8::new(idx).unwrap())
    }

    pub(crate) fn info(&self) -> &SymbolInfo {
        let idx = (self.0.get() - 1) as usize;
        &generated::SYMBOLS[idx]
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "`{}", self.name())
    }
}
