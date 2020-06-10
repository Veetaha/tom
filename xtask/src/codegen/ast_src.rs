//! Defines input for code generation process.

pub(crate) struct KindsSrc<'a> {
    pub(crate) punct: &'a [(&'a str, &'a str)],
    pub(crate) literals: &'a [&'a str],
    pub(crate) tokens: &'a [&'a str],
    pub(crate) nodes: &'a [&'a str],
}

pub(crate) const KINDS_SRC: KindsSrc = KindsSrc {
    punct: &[
        (".", "DOT"),
        (",", "COMMA"),
        ("[", "L_BRACK"),
        ("]", "R_BRACK"),
        ("{", "L_CURLY"),
        ("}", "R_CURLY"),
    ],
    literals: &[
        "INT_NUMBER",
        "FLOAT_NUMBER",
        "BASIC_STRING",
        "MULTILINE_BASIC_STRING",
        "DATE_TIME",
        "BOOL",
        "BARE_KEY_OR_NUMBER", // TODO: hmm, these two guys are weird...
        "BARE_KEY_OR_DATE",
    ],
    tokens: &[
        "ERROR",
        "BARE_KEY",
        "WHITESPACE",
        "COMMENT",
    ],
    nodes: &[
        "DOC",
        "ENTRY",
        "KEY",
        "VALUE",
        "ARRAY",
        "DICT",
        "TABLE_HEADER",
        "TABLE",
        "ARRAY_TABLE",
   ],
};

pub(crate) struct AstSrc<'a> {
    pub(crate) tokens: &'a [&'a str],
    pub(crate) nodes: &'a [AstNodeSrc<'a>],
    pub(crate) enums: &'a [AstEnumSrc<'a>],
}

pub(crate) struct AstNodeSrc<'a> {
    pub(crate) doc: &'a [&'a str],
    pub(crate) name: &'a str,
    pub(crate) traits: &'a [&'a str],
    pub(crate) fields: &'a [Field<'a>],
}

pub(crate) enum Field<'a> {
    Token(&'a str),
    Node { name: &'a str, src: FieldSrc<'a> },
}

pub(crate) enum FieldSrc<'a> {
    Shorthand,
    Optional(&'a str),
    Many(&'a str),
}

pub(crate) struct AstEnumSrc<'a> {
    pub(crate) doc: &'a [&'a str],
    pub(crate) name: &'a str,
    pub(crate) traits: &'a [&'a str],
    pub(crate) variants: &'a [&'a str],
}

macro_rules! ast_nodes {
    ($(
        $(#[doc = $doc:expr])+
        struct $name:ident$(: $($trait:ident),*)? {
            $($field_name:ident $(![$token:tt])? $(: $ty:tt)?),*$(,)?
        }
    )*) => {
        [$(
            AstNodeSrc {
                doc: &[$($doc),*],
                name: stringify!($name),
                traits: &[$($(stringify!($trait)),*)?],
                fields: &[
                    $(field!($(T![$token])? $field_name $($ty)?)),*
                ],

            }
        ),*]
    };
}

macro_rules! field {
    (T![$token:tt] T) => {
        Field::Token(stringify!($token))
    };
    ($field_name:ident) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Shorthand }
    };
    ($field_name:ident [$ty:ident]) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Many(stringify!($ty)) }
    };
    ($field_name:ident $ty:ident) => {
        Field::Node { name: stringify!($field_name), src: FieldSrc::Optional(stringify!($ty)) }
    };
}

macro_rules! ast_enums {
    ($(
        $(#[doc = $doc:expr])+
        enum $name:ident $(: $($trait:ident),*)? {
            $($variant:ident),*$(,)?
        }
    )*) => {
        [$(
            AstEnumSrc {
                doc: &[$($doc),*],
                name: stringify!($name),
                traits: &[$($(stringify!($trait)),*)?],
                variants: &[$(stringify!($variant)),*],
            }
        ),*]
    };
}

// TODO: make these AstToken ?
// n("Number").text(),
// n("Bool").text(),
// n("DateTime").text(),
// n("BareKey").text(),

pub(crate) const AST_SRC: AstSrc = AstSrc {
    tokens: &["Whitespace", "Comment"],
    nodes: &ast_nodes! {
        /// The entire toml source file (i.e. **the** top-level node)
        ///
        /// [Reference](https://github.com/toml-lang/toml)
        struct Doc: EntriesOwner {
            tables: [Table],
            array_tables: [ArrayTable],
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
        struct Table: EntriesOwner, TableHeaderOwner {}

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
        struct ArrayTable: EntriesOwner, TableHeaderOwner {}

        /// Toml table header
        ///
        /// ```toml
        /// <| [header.'foo'."bar"] |>
        /// <| [[header.'foo'."bar"]] |>
        /// ```
        ///
        /// [Reference](https://github.com/toml-lang/toml#table)
        struct TableHeader: KeysOwner {}

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
        struct Entry: KeysOwner {
            Value, T![=],
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
        struct Array {
            values: [Value]
        }

        /// Inline table literal
        ///
        /// ```toml
        /// name = <| { first = "Tom", last = "Preston-Werner" } |>
        /// ```
        ///
        /// [Reference](https://github.com/toml-lang/toml#user-content-inline-table)
        struct Dict: EntriesOwner {} // TODO: rename to InlineTable?
    },
    enums: &ast_enums! {
        // TODO: add manually?
        // n("StringLit")
        // .symbols(&[
        //     "BASIC_STRING",
        //     "MULTILINE_BASIC_STRING",
        //     "LITERAL_STRING",
        //     "MULTILINE_LITERAL_STRING",
        // ])

        enum StringLit {
            BASIC_STRING,
            MULTILINE_BASIC_STRING,
            LITERAL_STRING,
            MULTILINE_LITERAL_STRING,
        }

        enum Key {
            BareKey,
            StringLit,
        }

        enum Value {
            Array,
            Dict,
            Number,
            Bool,
            DateTime,
            StringLit,
        }

        // n("Key").kinds(&["StringLit", "BareKey"]),
        // n("Value").kinds(&["Array", "Dict", "Number", "Bool", "DateTime", "StringLit"]),
    },
};
