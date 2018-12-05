extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate clap;

use std::fs::File;
use std::io::Read;

use clap::{App, Arg};
use proc_macro2::{TokenStream, Span};
use syn::{Attribute, Path, PathArguments, ItemTrait, PathSegment, Ident};
use syn::token::{Pound, Bracket, Colon2};
use syn::AttrStyle::Outer;
use syn::punctuated::Punctuated;

fn main() {
    let matches = App::new("owasm-derive")
        .arg(Arg::with_name("code")
             .index(1)
             .required(true)
             .help("Path to contract code"))
        .get_matches();
    let mut file = File::open(matches.value_of("code").unwrap()).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code);

    let ast = syn::parse_file(&code).unwrap();

    let mut punctuated: Punctuated<PathSegment, Colon2>= Punctuated::new();
    punctuated.push_value(PathSegment {
        ident: Ident::new("owasm_abi_derive", Span::call_site()),
        arguments: PathArguments::None
    });
    punctuated.push_punct(Colon2::default());
    punctuated.push_value(PathSegment {
        ident: Ident::new("contract", Span::call_site()),
        arguments: PathArguments::None
    });

    let contract_attribute = Attribute {
        pound_token: Pound::default(),
        style: Outer,
        bracket_token: Bracket::default(),
        path: Path {
            leading_colon: None,
            segments: punctuated
        },
        tts: TokenStream::new(),
    };

    let mut traits_to_method_sigs: Vec<(String, Vec<TokenStream>)> =
        ast.items.iter().filter_map(|item| match item {
            syn::Item::Trait(m) => {
                if m.attrs.contains(&contract_attribute) {
                    let trait_name = m.ident.to_string();

                    let method_sigs = m.items.iter().filter_map(|item| {
                        match item {
                            syn::TraitItem::Method(m) => {
                                let msig = &m.sig;
                                let mattrs = &m.attrs;
                                Some(quote! {
                                    #(#mattrs)*
                                    #msig;
                                })
                            }
                            _ => None,
                        }
                    }).collect();
                    return Some((trait_name, method_sigs));
                }
                None
            }
            _ => None,
        }).collect();

    let mut contract_interface: String = "".to_owned();

    for (trait_name, method_sigs) in traits_to_method_sigs {
        contract_interface.push_str(&trait_name);
        contract_interface.push_str("\n");
        for method_sig in method_sigs {
            contract_interface.push_str(&method_sig.to_string());
            contract_interface.push_str("\n");
        }
    }

    println!("{}", contract_interface);

/*
    for tr in traits {
        if tr.attrs.contains(&contract_attribute) {
            println!("{:?}", tr.ident);

            for item in &tr.items {
                match item {
                    syn::TraitItem::Method(m) => {
                        println!("{:?}", m.sig);
                    }
                    _ => println!("not method"),
                }
            }
        }
    }
*/
}
