//! This module generates AST data type used by tom.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of newtype
//! wrappers around `SyntaxNode` which implement `tom_syntax::AstNode`.

use std::{collections::HashSet, fmt::Write};

use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};

use crate::{
    codegen::ast_src::{AstSrc, Field, FieldSrc, KindsSrc, AST_SRC, KINDS_SRC},
    codegen::{self, verify_or_overwrite, Mode},
    project_root_dir, Result,
};
use codegen::ast_src::AstEnumSrc;

pub fn generate_syntax(mode: Mode) -> Result<()> {
    let syntax_kinds_out_file = project_root_dir().join(codegen::SYNTAX_KINDS);
    let contents = generate_syntax_kinds(KINDS_SRC)?;
    verify_or_overwrite(mode, &syntax_kinds_out_file, &contents)?;

    let ast_tokens_out_file = project_root_dir().join(codegen::AST_TOKENS);
    let contents = generate_tokens(AST_SRC)?;
    verify_or_overwrite(mode, &ast_tokens_out_file, &contents)?;

    let ast_nodes_out_file = project_root_dir().join(codegen::AST_NODES);
    let contents = generate_nodes(KINDS_SRC, AST_SRC)?;
    verify_or_overwrite(mode, &ast_nodes_out_file, &contents)?;

    Ok(())
}

fn generate_tokens(grammar: AstSrc<'_>) -> Result<String> {
    let (token_defs, token_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .tokens
        .iter()
        .map(|token| {
            let name = format_ident!("{}", token);
            let kind = format_ident!("{}", to_upper_snake_case(token));
            (
                quote! {
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxToken,
                    }
                },
                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            std::fmt::Display::fmt(&self.syntax, f)
                        }
                    }
                    impl AstToken for #name {
                        fn can_cast(kind: SyntaxKind) -> bool { kind == #kind }
                        fn cast(syntax: SyntaxToken) -> Option<Self> {
                            if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                        }
                        fn syntax(&self) -> &SyntaxToken { &self.syntax }
                    }
                },
            )
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls) = generate_enums(grammar.token_enums, EnumKind::Token);

    let mut docs = grammar.token_enums.iter().map(|it| it.doc);

    let ast = quote! {
        use crate::{SyntaxKind::{self, *}, SyntaxToken, ast::AstToken};
        #(#token_defs)*
        #(#enum_defs)*

        #(#token_boilerplate_impls)*
        #(#enum_boilerplate_impls)*
    }
    .to_string();

    let pretty = crate::reformat(replace_docs_placeholder_with_pretty_comment(
        &ast, &mut docs,
    ))?;
    // .replace("#[derive", "\n#[derive"); // TODO: is this needed?
    Ok(pretty)
}

fn into_ident(ident_name: impl quote::IdentFragment) -> proc_macro2::Ident {
    format_ident!("{}", ident_name)
}

enum EnumKind {
    Node,
    Token,
}

fn generate_enums(
    enums: &[AstEnumSrc<'_>],
    enum_kind: EnumKind,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    enums
        .iter()
        .map(|en| {
            let name = into_ident(en.name);

            let (ast_trait, syntax_type) = match enum_kind {
                EnumKind::Node => (into_ident("AstNode"), into_ident("SyntaxNode")),
                EnumKind::Token => (into_ident("AstToken"), into_ident("SyntaxToken")),
            };

            let variants: Vec<_> = en.variants.iter().map(into_ident).collect();

            let kinds: Vec<_> = variants
                .iter()
                .map(|it| to_upper_snake_case(&it.to_string()))
                .map(into_ident)
                .collect();

            let traits = en
                .traits
                .iter()
                .map(into_ident)
                .map(|trait_name| quote!(impl ast::#trait_name for #name {}));

            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub enum #name {
                        #(#variants(#variants),)*
                    }
                    #(#traits)*
                },
                quote! {
                    impl #ast_trait for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            match kind {
                                #(#kinds)|* => true,
                                _ => false,
                            }
                        }
                        fn cast(syntax: #syntax_type) -> Option<Self> {
                            let res = match syntax.kind() {
                                #(#kinds => #name::#variants(#variants { syntax }),)*
                                _ => return None,
                            };
                            Some(res)
                        }
                        fn syntax(&self) -> &#syntax_type {
                            match self {
                                #(#name::#variants(it) => &it.syntax,)*
                            }
                        }
                    }
                    #(
                        impl From<#variants> for #name {
                            fn from(val: #variants) -> #name {
                                #name::#variants(val)
                            }
                        }
                    )*
                },
            )
        })
        .unzip()
}

fn generate_nodes(kinds: KindsSrc<'_>, grammar: AstSrc<'_>) -> Result<String> {
    let (node_defs, node_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let kind = format_ident!("{}", to_upper_snake_case(&node.name));
            let traits = node.traits.iter().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl ast::#trait_name for #name {})
            });

            let methods = node.fields.iter().map(|field| {
                let method_name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        pub fn #method_name(&self) -> AstChildren<#ty> {
                            support::children(&self.syntax)
                        }
                    }
                } else {
                    if let Some(token_kind) = field.token_kind() {
                        quote! {
                            pub fn #method_name(&self) -> Option<#ty> {
                                support::token(&self.syntax, #token_kind)
                            }
                        }
                    } else {
                        quote! {
                            pub fn #method_name(&self) -> Option<#ty> {
                                support::child(&self.syntax)
                            }
                        }
                    }
                }
            });
            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }

                    #(#traits)*

                    impl #name {
                        #(#methods)*
                    }
                },
                quote! {
                    impl AstNode for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            kind == #kind
                        }
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                        }
                        fn syntax(&self) -> &SyntaxNode { &self.syntax }
                    }
                },
            )
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls) = generate_enums(grammar.node_enums, EnumKind::Node);

    let enum_names = grammar.node_enums.iter().map(|it| it.name);
    let node_names = grammar.nodes.iter().map(|it| it.name);

    let display_impls = enum_names
        .chain(node_names.clone())
        .map(|it| format_ident!("{}", it))
        .map(|name| {
            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        std::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            }
        });

    let defined_nodes: HashSet<_> = node_names.collect();

    for node in kinds
        .nodes
        .iter()
        .map(|kind| to_pascal_case(kind))
        .filter(|name| !defined_nodes.contains(name.as_str()))
    {
        eprintln!("Warning: node {} not defined in ast source", node);
    }

    let ast = quote! {
        use crate::{
            SyntaxNode, SyntaxToken, SyntaxKind::{self, *},
            ast::{self, AstNode, AstChildren, support},
            T,
        };

        #(#node_defs)*
        #(#enum_defs)*
        #(#node_boilerplate_impls)*
        #(#enum_boilerplate_impls)*
        #(#display_impls)*
    };

    let ast = ast
        .to_string()
        .replace("T ! [ ", "T![")
        .replace(" ] )", "])");

    let mut docs = grammar
        .nodes
        .iter()
        .map(|it| it.doc)
        .chain(grammar.node_enums.iter().map(|it| it.doc));
    let res = replace_docs_placeholder_with_pretty_comment(&ast, &mut docs);

    let pretty = crate::reformat(res)?;
    Ok(pretty)
}

fn replace_docs_placeholder_with_pretty_comment(
    code: &str,
    docs: &mut dyn Iterator<Item = &[&str]>,
) -> String {
    let mut acc = String::new();
    for chunk in code.split("# [ pretty_doc_comment_placeholder_workaround ]") {
        acc.push_str(chunk);
        if let Some(doc) = docs.next() {
            for line in doc {
                writeln!(acc, "///{}", line).unwrap();
            }
        }
    }
    acc
}

fn generate_syntax_kinds(grammar: KindsSrc<'_>) -> Result<String> {
    let punctuation_values = grammar.punct.iter().map(|(token, _name)| {
        if "{}[]()".contains(token) {
            let c = token.chars().next().unwrap();
            quote! { #c }
        } else {
            let cs = token.chars().map(|c| Punct::new(c, Spacing::Joint));
            quote! { #(#cs)* }
        }
    });
    let punctuation: Vec<_> = grammar
        .punct
        .iter()
        .map(|(_token, name)| format_ident!("{}", name))
        .collect();
    let literals: Vec<_> = grammar
        .literals
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect();
    let tokens: Vec<_> = grammar
        .tokens
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect();
    let nodes: Vec<_> = grammar
        .nodes
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect();
    let contextual_tokens: Vec<_> = grammar
        .contextual_tokens
        .iter()
        .map(|name| format_ident!("{}", name))
        .collect();

    let ast = quote! {
        #![allow(bad_style, missing_docs, unreachable_pub)]
        /// The kind of syntax node, e.g. `INT_NUMBER`, `LITERAL_LINE_STRING`, or `COMMA`.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[repr(u16)]
        pub enum SyntaxKind {
            // Technical SyntaxKinds: they appear temporally during parsing,
            // but never end up in the final tree
            #[doc(hidden)]
            TOMBSTONE,
            #[doc(hidden)]
            EOF,
            #(#punctuation,)*
            #(#literals,)*
            #(#tokens,)*
            #(#contextual_tokens,)*
            #(#nodes,)*

            // Technical kind so that we can cast from u16 safely
            #[doc(hidden)]
            __LAST,
        }
        use self::SyntaxKind::*;

        impl SyntaxKind {
            // TODO:
            // pub fn is_punct(self) -> bool {
            //     match self {
            //         #(#punctuation)|* => true,
            //         _ => false,
            //     }
            // }

            // pub fn is_literal(self) -> bool {
            //     match self {
            //         #(#literals)|* => true,
            //         _ => false,
            //     }
            // }

            // pub fn from_char(c: char) -> Option<SyntaxKind> {
            //     let tok = match c {
            //         #(#single_byte_tokens_values => #single_byte_tokens,)*
            //         _ => return None,
            //     };
            //     Some(tok)
            // }
        }

        #[macro_export]
        macro_rules! T {
            #([#punctuation_values] => { $crate::SyntaxKind::#punctuation };)*
            [lifetime] => { $crate::SyntaxKind::LIFETIME };
            [ident] => { $crate::SyntaxKind::IDENT };
        }
    };

    crate::reformat(ast)
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_lowercase());
    }
    buf
}

fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else {
            buf.push(c.to_ascii_lowercase());
        }
    }
    buf
}

impl Field<'_> {
    fn is_many(&self) -> bool {
        matches!(self, Field::Node { src: FieldSrc::Many(_), .. })
    }
    fn token_kind(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            Field::Token(token) => {
                let token: proc_macro2::TokenStream = token.parse().unwrap();
                Some(quote! { T![#token] })
            }
            _ => None,
        }
    }
    fn method_name(&self) -> proc_macro2::Ident {
        match self {
            Field::Token(name) => {
                let name = match *name {
                    "'{'" => "l_curly",
                    "'}'" => "r_curly",
                    "'['" => "l_brack",
                    "']'" => "r_brack",
                    "=" => "eq",
                    "." => "dot",
                    ":" => "colon",
                    _ => name,
                };
                format_ident!("{}_token", name)
            }
            Field::Node { name, src } => match src {
                FieldSrc::Shorthand => format_ident!("{}", to_lower_snake_case(name)),
                _ => format_ident!("{}", name),
            },
        }
    }
    fn ty(&self) -> proc_macro2::Ident {
        match self {
            Field::Token(_) => format_ident!("SyntaxToken"),
            Field::Node { name, src } => match src {
                FieldSrc::Optional(ty) | FieldSrc::Many(ty) => format_ident!("{}", ty),
                FieldSrc::Shorthand => format_ident!("{}", name),
            },
        }
    }
}
