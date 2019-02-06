#![recursion_limit = "192"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate serde_derive;

mod rustfmt;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn read_file(p: &Path) -> String {
    fs::read_to_string(p).expect(&format!("Error: could not read {}", p.to_str().unwrap()))
}

#[derive(Serialize, Deserialize, Debug)]
struct TomlProject {
    name: String,
    version: String,
    authors: Vec<String>,
    edition: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TomlManifest {
    package: TomlProject,
    #[serde(serialize_with = "toml::ser::tables_last")]
    dependencies: BTreeMap<String, toml::Value>,
}

fn gen_cargo_toml(cargo_path: &Path) -> String {
    let mut mf: TomlManifest = toml::from_str(&read_file(cargo_path)).unwrap();
    mf.package.name += "-abi";
    mf.dependencies = mf
        .dependencies
        .into_iter()
        .filter(|(name, _version)| name.starts_with("owasm-"))
        .collect();
    toml::to_string(&mf).unwrap()
}

fn main() {
    let matches = app_from_crate!()
        .arg(
            clap::Arg::with_name("contract-crate")
                .index(1)
                .required(true)
                .help("Path to contract crate"),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .help("Output directory")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("force")
                .short("f")
                .help("Overwrite existing ABI crate."),
        )
        .get_matches();

    let crate_path = Path::new(matches.value_of("contract-crate").unwrap());
    let lib_path = crate_path.join("src/lib.rs");
    let cargo_path = crate_path.join("Cargo.toml");

    let mut abi_crate_path;
    if matches.is_present("output") {
        abi_crate_path = PathBuf::from(matches.value_of("output").unwrap());
        if !abi_crate_path.exists() {
            fs::create_dir_all(&abi_crate_path).expect(&format!(
                "Failed to create output directory {}",
                abi_crate_path.to_str().unwrap()
            ));
        }
    } else {
        abi_crate_path = std::env::current_dir().unwrap();
    }
    abi_crate_path.push(format!(
        "{}_abi",
        crate_path.file_name().unwrap().to_str().unwrap()
    ));

    let abi_crate_opts = cargo::ops::NewOptions::new(
        Some(cargo::ops::VersionControl::Git),
        false, /* bin */
        true,  /* lib */
        abi_crate_path.clone(),
        None, /* name */
        None, /* edition */
        None, /* registry */
    );
    let did_init_crate = cargo::ops::init(
        &abi_crate_opts.unwrap(),
        &cargo::util::Config::default().unwrap(), /* config */
    );

    if did_init_crate.is_err() && !matches.is_present("force") {
        println!("Generated ABI crate already exists. Pass `-f` to overwrite.");
        return;
    }

    let ast = syn::parse_file(&read_file(&lib_path)).expect("lib.rs has syntax errors.");
    let contract_trait = ast
        .items
        .iter()
        .filter_map(|itm| match itm {
            syn::Item::Trait(t) => t
                .attrs
                .iter()
                .find(|attr| match attr.path.segments.last() {
                    Some(syn::punctuated::Pair::End(e)) => e.ident == "contract",
                    _ => false,
                })
                .and(Some(t)),
            _ => None,
        })
        .nth(0)
        .expect("Could not find trait annotated with `owasm_abi_derive::contract`");

    let contract = owasm_abi_utils::Contract::new(&contract_trait);

    let trait_name = contract.trait_name;
    let contract_ep = contract.endpoint_name;
    let contract_client = contract.client_name;
    let method_sigs = contract.method_sigs;

    let abi_lib = quote! {
      extern crate owasm_abi;
      extern crate owasm_abi_derive;
      extern crate owasm_ethereum;

      #[owasm_abi_derive::eth_abi(#contract_ep, #contract_client)]
      pub trait #trait_name {
        #(#method_sigs)*
      }
    };

    let (output, error) = rustfmt::format(abi_lib.to_string());
    if !error.is_empty() && error.split("\n").any(|line| !line.starts_with("Warning:")) {
        println!("Failed to format generated code. Error: {}", error);
        return;
    }
    fs::write(abi_crate_path.join("src/lib.rs").as_path(), output).unwrap();

    fs::write(
        abi_crate_path.join("Cargo.toml").as_path(),
        gen_cargo_toml(cargo_path.as_path()),
    )
    .unwrap();

    println!(
        "ABI crate for {} created at {}",
        trait_name,
        abi_crate_path.to_str().unwrap()
    );
}
