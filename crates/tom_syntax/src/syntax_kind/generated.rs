//! Generated file, do not edit by hand, see `xtask/src/codegen`

#![allow(bad_style, missing_docs, unreachable_pub)]
#[doc = r" The kind of syntax node, e.g. `INT_NUMBER`, `LITERAL_LINE_STRING`, or `COMMA`."]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    DOT,
    COMMA,
    L_BRACK,
    R_BRACK,
    L_CURLY,
    R_CURLY,
    EQ,
    PLUS,
    COLON,
    INT,
    FLOAT,
    STRING,
    LITERAL_LINE_STRING,
    LITERAL_MULTILINE_STRING,
    BASIC_LINE_STRING,
    BASIC_MULTILINE_STRING,
    DATE_TIME,
    TRUE,
    FALSE,
    ERROR,
    BARE_KEY,
    WHITESPACE,
    COMMENT,
    NEWLINE,
    BARE_KEY_LIKE,
    DOC,
    ENTRY,
    KEY,
    VALUE,
    ARRAY,
    DICT,
    TABLE_HEADER,
    TABLE,
    ARRAY_TABLE,
    ATOM_LITERAL,
    #[doc(hidden)]
    __LAST,
}
use self::SyntaxKind::*;
impl SyntaxKind {}
#[macro_export]
macro_rules ! T { [ . ] => { $ crate :: SyntaxKind :: DOT } ; [ , ] => { $ crate :: SyntaxKind :: COMMA } ; [ '[' ] => { $ crate :: SyntaxKind :: L_BRACK } ; [ ']' ] => { $ crate :: SyntaxKind :: R_BRACK } ; [ '{' ] => { $ crate :: SyntaxKind :: L_CURLY } ; [ '}' ] => { $ crate :: SyntaxKind :: R_CURLY } ; [ = ] => { $ crate :: SyntaxKind :: EQ } ; [ + ] => { $ crate :: SyntaxKind :: PLUS } ; [ : ] => { $ crate :: SyntaxKind :: COLON } ; [ lifetime ] => { $ crate :: SyntaxKind :: LIFETIME } ; [ ident ] => { $ crate :: SyntaxKind :: IDENT } ; }
