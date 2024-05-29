mod oscdsl;
mod syntax;
mod syntax_spec;

use std::{
    env,
    path::{Path, PathBuf},
};

use clap::{command, Command};
use quote::{format_ident, quote};
use std::fmt::Write;
use syntax::{TokenData, DSL_TOKENS};
use ungrammar::{Grammar, Rule};
use xshell::{cmd, Shell};

fn project_root() -> PathBuf {
    let dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());
    PathBuf::from(dir).parent().unwrap().to_owned()
}

fn generate_char_set() {
    let tmp_dir =
        tempdir::TempDir::new("oscelas-xtask").expect("Failed to create a temporary directory");
    let unic_ver = "15.1.0";
    let ucd_dir = tmp_dir.path().join("UCD");
    let gen_dir = project_root().join("oscelas/src/chars/generated");

    let sh = Shell::new().expect("Failed to create a shell");
    cmd!(
        sh,
        "curl -L https://www.unicode.org/Public/zipped/{unic_ver}/UCD.zip -o {ucd_dir}.zip"
    )
    .run()
    .expect("Failed to download UCD.zip");

    cmd!(sh, "unzip -q {ucd_dir}.zip -d {ucd_dir}")
        .run()
        .expect("Failed to unzip UCD.zip");

    do_generate_char_set(
        &ucd_dir,
        &gen_dir.join("id_start_char_set.rs"),
        "ID_START_CHAR_SET",
        &["Lu", "Ll", "Lt", "Lm", "Lo", "Nl"],
    );
    do_generate_char_set(
        &ucd_dir,
        &gen_dir.join("id_char_set.rs"),
        "ID_CHAR_SET",
        &["Lu", "Ll", "Lt", "Lm", "Lo", "Nl", "Mc", "Mn", "Nd", "Pc"],
    );
}

fn do_generate_char_set(ucd_path: &Path, out_path: &Path, var_name: &str, categories: &[&str]) {
    let sh = Shell::new().expect("Failed to create a shell");
    let categories_str = categories.join(",");

    let code = cmd!(sh, "ucd-generate general-category {ucd_path} --trie-set --include {categories_str} --combined --name {var_name}")
        .read()
        .expect("Failed to generate char set");

    sh.write_file(out_path, code)
        .expect("Failed to write char set");
}

fn generate_dsl_syntax() {
    let grammar = include_str!("oscdsl.ungram");
    let grammar = grammar
        .parse::<Grammar>()
        .expect("Failed to parse the grammar");

    generate_dsl_syntax_kinds(&DSL_TOKENS, &grammar);
    generate_dsl_syntax_node(&grammar);
}

fn generate_dsl_syntax_kinds(data: &TokenData, grammer: &Grammar) {
    let gen_dir = project_root().join("oscelas/src/syntax/generated");

    let sh = Shell::new().expect("Failed to create a shell");

    let puncts = data
        .punct
        .iter()
        .map(|(_, name)| format_ident!("{name}"))
        .collect::<Vec<_>>();

    let punct_strs = data
        .punct
        .iter()
        .map(|(punct, _)| punct)
        .collect::<Vec<_>>();

    let keywords = data
        .keyword
        .iter()
        .map(|kw| kw.to_ascii_uppercase())
        .map(|kw| format_ident!("{kw}_KW"))
        .collect::<Vec<_>>();

    let keywords_strs = data.keyword;

    let literals = data
        .literal
        .iter()
        .map(|lit| format_ident!("{lit}"))
        .collect::<Vec<_>>();

    let tokens = data
        .token
        .iter()
        .map(|tok| format_ident!("{tok}"))
        .collect::<Vec<_>>();

    let nodes = grammer
        .iter()
        .map(|node| format_ident!("{}", to_upper_snake_case(&grammer[node].name)))
        .collect::<Vec<_>>();

    let lists = grammer
        .iter()
        .filter(|node| grammer[*node].name.ends_with("List"))
        .map(|node| format_ident!("{}", to_upper_snake_case(&grammer[node].name)))
        .collect::<Vec<_>>();

    let code = quote! {
        #![allow(bad_style, missing_docs, unreachable_pub)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum OscDslSyntaxKind {
            #[doc(hidden)]
            TOMBSTONE,
            EOF,
            #(#puncts,)*
            #(#keywords,)*
            #(#literals,)*
            #(#tokens,)*
            #(#nodes,)*

            #[doc(hidden)]
            __LAST,
        }

        use self::OscDslSyntaxKind::*;
        impl OscDslSyntaxKind {
            pub fn is_list(self) -> bool {
                matches!(self, #(#lists)|*)
            }

            pub fn to_string(self) -> Option<&'static str> {
                match self {
                    #(#puncts => Some(#punct_strs),)*
                    #(#keywords => Some(#keywords_strs),)*
                    _ => None,
                }
            }
        }
    };

    let code = cmd!(sh, "rustfmt --emit stdout")
        .stdin(code.to_string())
        .read()
        .expect("Failed to format the code");

    sh.write_file(gen_dir.join("kind.rs"), code)
        .expect("Failed to write kind.rs");
}

fn generate_dsl_syntax_node(grammer: &Grammar) {
    let gen_dir = project_root().join("oscelas/src/syntax/generated");

    let sh = Shell::new().expect("Failed to create a shell");

    let mut code = String::new();
    write!(
        code,
        "{}",
        quote! {
            use super::OscDslSyntaxKind;
            use crate::syntax::OscDslLanguage;
            use biome_rowan::{AstNode, RawSyntaxKind, SyntaxKindSet, SyntaxNode};
        }
    )
    .unwrap();

    for node in grammer.iter() {
        let node = &grammer[node];
        let name = format_ident!("{}", node.name);
        let kind = format_ident!("{}", to_upper_snake_case(&node.name));

        let fragment = quote! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #name {
                node: SyntaxNode<OscDslLanguage>,
            }

            impl #name {
                pub const unsafe fn new_unchecked(node: SyntaxNode<OscDslLanguage>) -> Self {
                    Self { node }
                }
            }

            impl AstNode for #name {
                type Language = OscDslLanguage;

                const KIND_SET: SyntaxKindSet<Self::Language> =
                    SyntaxKindSet::from_raw(RawSyntaxKind(OscDslSyntaxKind::#kind as u16));

                fn can_cast(kind: OscDslSyntaxKind) -> bool {
                    kind == OscDslSyntaxKind::#kind
                }

                fn cast(node: SyntaxNode<OscDslLanguage>) -> Option<Self> {
                    Self::can_cast(node.kind()).then(|| unsafe { Self::new_unchecked(node) })
                }

                fn syntax(&self) -> &SyntaxNode<OscDslLanguage> {
                    &self.node
                }

                fn into_syntax(self) -> SyntaxNode<OscDslLanguage> {
                    self.node
                }
            }
        };

        write!(code, "{fragment}").unwrap();
    }

    let code = cmd!(sh, "rustfmt --emit stdout")
        .stdin(code)
        .read()
        .expect("Failed to format the code");

    sh.write_file(gen_dir.join("node.rs"), code)
        .expect("Failed to write node.rs");
}

fn to_upper_snake_case(s: &str) -> String {
    // `s` is expected to be represented in UpperCamelCase
    let mut result = String::new();
    let mut first = true;
    for c in s.chars() {
        if c.is_ascii_uppercase() && !first {
            result.push('_');
        }
        first = false;
        result.push(c.to_ascii_uppercase());
    }
    result
}

fn main() {
    let matches = command!()
        .subcommand(Command::new("generate_char_set"))
        .subcommand(Command::new("generate_dsl_syntax"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("generate_char_set") {
        generate_char_set();
    } else if let Some(_) = matches.subcommand_matches("generate_dsl_syntax") {
        generate_dsl_syntax();
    }
}
