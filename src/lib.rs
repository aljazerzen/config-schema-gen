use std::{env, fs};

use proc_macro::TokenStream;
use codegen::{Scope, Trait, Type};
use jsl::SerdeSchema;

#[proc_macro]
pub fn embed_typed_config(input: TokenStream) -> TokenStream {

    quote!(
        
    )

}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("usage: config-schema-gen [config-schema.json] > output-file.rs");
    }

    let schema_filename = args[1].clone();
    let schema_file = fs::read_to_string(schema_filename.clone())
        .unwrap_or_else(|e| panic!("cannot read {:}: {:}", schema_filename, e));

    let config_schema: SerdeSchema =
        serde_json::from_str(&schema_file).expect("cannot parse JSON shema");

    let mut scope = Scope::new();

    let trt = scope.new_trait("TypedConfig").vis("pub");

    trt.new_fn("get_bool")
        .arg_self()
        .arg("key", "&str")
        .ret(Type::new("Option").generic("bool").clone());
    trt.new_fn("get_int")
        .arg_self()
        .arg("key", "&str")
        .ret(Type::new("Option").generic("i64").clone());
    trt.new_fn("get_str")
        .arg_self()
        .arg("key", "&str")
        .ret(Type::new("Option").generic("String").clone());

    println!(
        "// This file was generated with config-schama-gen from {:}",
        schema_filename
    );

    gen_getters(trt, "".to_string(), config_schema);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hello.rs");
    fs::write(
        &dest_path,
        scope.to_string()
    ).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

fn gen_getters(trt: &mut Trait, prefix: String, schema: SerdeSchema) {
    let typ = schema.typ.unwrap_or_else(|| "string".to_string());

    if typ == "object" {
        for p in schema.props.expect("missing properties") {
            let prefix = if prefix == "" {
                p.0
            } else {
                format!("{:}.{:}", prefix, p.0)
            };

            gen_getters(trt, prefix, p.1)
        }
    } else {
        let name = prefix.clone().replace('.', "_").replace('-', "_");

        let funct = trt
            .new_fn(format!("{:}", name).as_str())
            .arg_self();

        let default = schema.extra.get("default");

        if typ == "string" {
            if let Some(default) = default {
                funct.ret("String").line(format!(
                    "self.get_str(\"{:}\").unwrap_or_else(|| {:}.to_string())",
                    prefix, default
                ));
            } else {
                funct
                    .ret(Type::new("Option").generic("String").clone())
                    .line(format!("self.get_str(\"{:}\")", prefix));
            }
        } else if typ == "boolean" {
            if let Some(default) = default {
                funct.ret("bool").line(format!(
                    "self.get_bool(\"{:}\").unwrap_or({:})",
                    prefix, default
                ));
            } else {
                funct
                    .ret(Type::new("Option").generic("bool").clone())
                    .line(format!("self.get_bool(\"{:}\")", prefix));
            }
        } else if typ == "integer" {
            if let Some(default) = default {
                funct.ret("i64").line(format!(
                    "self.get_int(\"{:}\").unwrap_or({:})",
                    prefix, default
                ));
            } else {
                funct
                    .ret(Type::new("Option").generic("i64").clone())
                    .line(format!("self.get_int(\"{:}\")", prefix));
            }
        } else {
            funct.line(format!("// TODO: cannot generate getter for type {:}", typ).to_string());
        }
    }
}
