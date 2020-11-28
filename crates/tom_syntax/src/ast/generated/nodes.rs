//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::{
    SyntaxNode, SyntaxToken,
    SyntaxKind::{self, *},
    ast::{self, AstNode, AstChildren, support},
    T,
};
/// The entire toml source file (i.e. **the** top-level node)
///
/// [Reference](https://github.com/toml-lang/toml)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Doc {
    pub(crate) syntax: SyntaxNode,
}
impl ast::EntriesOwner for Doc {}
impl Doc {
    pub fn tables(&self) -> AstChildren<Table> {
        support::children(&self.syntax)
    }
    pub fn array_tables(&self) -> AstChildren<ArrayTable> {
        support::children(&self.syntax)
    }
}
/// Table (non-top-level one)
///
/// ```toml
/// <|
/// [header.'foo'."bar"]
/// entry1 = 32
/// entry2 = "baz"
/// |>
/// <|
/// [header2]
/// entry1 = false
/// |>
/// ```
///
/// [Reference](https://github.com/toml-lang/toml#table)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Table {
    pub(crate) syntax: SyntaxNode,
}
impl ast::EntriesOwner for Table {}
impl ast::TableHeaderOwner for Table {}
impl Table {}
/// Array of tables item (not to be confused with `Array`)
///
/// ```toml
/// <|
/// [[header.'foo'."bar"]]
/// entry1 = 32
/// entry2 = "baz"
/// |>
/// <|
/// [[header2]]
/// entry1 = false
/// |>
/// ```
///
/// [Reference](https://github.com/toml-lang/toml#user-content-array-of-tables)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayTable {
    pub(crate) syntax: SyntaxNode,
}
impl ast::EntriesOwner for ArrayTable {}
impl ast::TableHeaderOwner for ArrayTable {}
impl ArrayTable {}
/// Toml table header
///
/// ```toml
/// <| [header.'foo'."bar"] |>
/// <| [[header.'foo'."bar"]] |>
/// ```
///
/// [Reference](https://github.com/toml-lang/toml#table)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableHeader {
    pub(crate) syntax: SyntaxNode,
}
impl ast::KeysOwner for TableHeader {}
impl TableHeader {}
/// Entry in top-level doc, tables or inline tables
///
/// ```toml
/// <| top_level = 42 |>
/// <| inline_table = { <| a = 1 |>, <| c = { <| d = 3 |> } } |> |>
/// [table]
/// <| bar = 32 |>
/// ```
///
/// - [Reference](https://github.com/toml-lang/toml#table)
/// - [Reference](https://github.com/toml-lang/toml#user-content-inline-table)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entry {
    pub(crate) syntax: SyntaxNode,
}
impl ast::KeysOwner for Entry {}
impl Entry {
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
    pub fn eq_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![=])
    }
}
/// Array literal (not to be confused with `ArrayTable`)
///
/// ```toml
/// arr = <| [1, <| [42] |>, true, <| ["blah", <| [] |> ] |>, 1970-01-01,] |>
/// arr2 = <| [
///     <| [
///         <| ["hello", 'world'] |>
///     ] |>
/// ] |>
/// ```
///
/// [Reference](https://github.com/toml-lang/toml#user-content-array)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Array {
    pub(crate) syntax: SyntaxNode,
}
impl Array {
    pub fn values(&self) -> AstChildren<Value> {
        support::children(&self.syntax)
    }
}
/// Inline table literal
///
/// ```toml
/// name = <| { first = "Tom", last = "Preston-Werner" } |>
/// ```
///
/// [Reference](https://github.com/toml-lang/toml#user-content-inline-table)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dict {
    pub(crate) syntax: SyntaxNode,
}
impl ast::EntriesOwner for Dict {}
impl Dict {}
/// Represents an idivisible datum literal
/// ```toml
/// foo = <| 42 |>
/// bar = <| "bruh" |>
/// baz = <| true |>
/// bruh = <| 2020-05-13 |>
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomLiteral {
    pub(crate) syntax: SyntaxNode,
}
impl AtomLiteral {}
/// Represents any kind of an atomic table key token.
///
/// [<| foo |> . <| "bar" |>. <| 'baz' |>]
/// <| bruh |> = 42
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub(crate) syntax: SyntaxNode,
}
impl Key {}
/// Represents any kind of value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Array(Array),
    Dict(Dict),
    AtomLiteral(AtomLiteral),
}
impl AstNode for Doc {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DOC
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Table {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TABLE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ArrayTable {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARRAY_TABLE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for TableHeader {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TABLE_HEADER
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Entry {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENTRY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Array {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARRAY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Dict {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DICT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for AtomLiteral {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ATOM_LITERAL
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Key {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == KEY
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Value {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            ARRAY | DICT | ATOM_LITERAL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            ARRAY => Value::Array(Array { syntax }),
            DICT => Value::Dict(Dict { syntax }),
            ATOM_LITERAL => Value::AtomLiteral(AtomLiteral { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Value::Array(it) => &it.syntax,
            Value::Dict(it) => &it.syntax,
            Value::AtomLiteral(it) => &it.syntax,
        }
    }
}
impl From<Array> for Value {
    fn from(val: Array) -> Value {
        Value::Array(val)
    }
}
impl From<Dict> for Value {
    fn from(val: Dict) -> Value {
        Value::Dict(val)
    }
}
impl From<AtomLiteral> for Value {
    fn from(val: AtomLiteral) -> Value {
        Value::AtomLiteral(val)
    }
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Doc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrayTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TableHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Dict {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AtomLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
