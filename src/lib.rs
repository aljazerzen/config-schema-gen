use std::{env, fs, path::PathBuf};

use jsl::SerdeSchema;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, LitBool, LitInt, LitStr, parse_macro_input};

pub(crate) fn crate_root() -> PathBuf {
    let crate_root = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not present");
    PathBuf::from(crate_root)
}

#[proc_macro]
pub fn embed_typed_config(input: TokenStream) -> TokenStream {
    // find, read & parse schema file

    let schema_filename = if input.is_empty() {
        crate_root().join("config.schema.json")
    } else {
        let location: LitStr = parse_macro_input!(input);
        crate_root().join(location.value())
    };

    let schema_file = fs::read_to_string(schema_filename.clone())
        .unwrap_or_else(|e| panic!("cannot read {:?}: {:}", schema_filename, e));
    let config_schema: SerdeSchema =
        serde_json::from_str(&schema_file).expect("cannot parse JSON shema");

    // generate the TypedConfig trait

    let getters = gen_getters("".to_string(), config_schema);

    (quote! {
        pub trait TypedConfig {
            fn get_bool(self: &Self, key: &str) -> Option<bool>;

            fn get_int(self: &Self, key: &str) -> Option<i64>;

            fn get_str(self: &Self, key: &str) -> Option<String>;

            #getters
        }
    }).into()
}

fn gen_getters(prefix: String, schema: SerdeSchema) -> TokenStream2 {
    let typ = schema.typ.unwrap_or_else(|| "string".to_string());

    if typ == "object" {
        let mut props = Vec::new();
        for p in schema.props.expect("missing properties") {
            let prefix = if prefix == "" {
                p.0
            } else {
                format!("{:}.{:}", prefix, p.0)
            };

            props.push(gen_getters(prefix, p.1));
        }
        quote! { #(#props)* }
    } else {
        let name = prefix.clone().replace('.', "_").replace('-', "_");
        let name = Ident::new(&name, Span::call_site());

        let default = schema.extra.get("default");

        let (ret_type, line) = if typ == "string" {
            if let Some(default) = default {
                let default = LitStr::new(default.as_str().unwrap(), Span::call_site());
                (
                    quote! { String },
                    quote! { self.get_str(#prefix).unwrap_or_else(| | #default.to_string()) },
                )
            } else {
                (quote! { Option<String> }, quote! { self.get_str(#prefix) })
            }
        } else if typ == "boolean" {
            if let Some(default) = default {
                let default = LitBool::new(default.as_bool().unwrap(), Span::call_site());
                (
                    quote! { bool },
                    quote! { self.get_bool(#prefix).unwrap_or(#default) },
                )
            } else {
                (quote! { Option<bool> }, quote! { self.get_bool(#prefix) })
            }
        } else if typ == "integer" {
            if let Some(default) = default {
                let default = LitInt::new(&default.to_string(), Span::call_site());
                (
                    quote! { i64 },
                    quote! { self.get_int(#prefix).unwrap_or(#default) },
                )
            } else {
                (quote! { Option<i64> }, quote! { self.get_int(#prefix) })
            }
        } else {
            (quote! { () }, quote! {})
        };

        quote! {
            fn #name(&self) -> #ret_type {
                #line
            }
        }
    }
}
