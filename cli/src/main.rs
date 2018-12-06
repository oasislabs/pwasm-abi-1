#![recursion_limit = "192"]

extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate clap;

use std::fs::File;
use std::io::Read;

use clap::{App, Arg};
use proc_macro2::{TokenStream, Span, Group};
use syn::{Attribute, Path, PathArguments, ItemTrait, PathSegment, Ident};
use syn::token::{Pound, Bracket, Colon2};
use syn::AttrStyle::Outer;
use syn::punctuated::Punctuated;

macro_rules! format_ident {
  ($ident:expr, $fstr:expr) => {
      syn::Ident::new(&format!($fstr, $ident), $ident.span())
    };
}

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

    let mut traits_to_method_sigs: Vec<(&Ident, Vec<TokenStream>)> =
        ast.items.iter().filter_map(|item| match item {
            syn::Item::Trait(m) => {
                if m.attrs.contains(&contract_attribute) {
                    let trait_name = &m.ident;

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

    for (trait_name, method_sigs) in traits_to_method_sigs {
        let contract_endpoint = format_ident!(trait_name, "{}Endpoint");
        let contract_client = format_ident!(trait_name, "{}Client");
        let contract_struct = format_ident!(trait_name, "{}Inst");
        let contract_interface = format_ident!(trait_name, "{}Interface");

        let mut methods_stream = quote!();
        for method_sig in method_sigs {
            methods_stream.extend(method_sig);
        }
        let methods_group = Group::new(proc_macro2::Delimiter::Brace, methods_stream);
        let mut trait_stream = quote! {
            extern crate owasm_abi;
            extern crate owasm_abi_derive;
            extern crate owasm_ethereum;

            use owasm_abi::eth::EndpointInterface;
            use owasm_abi::types::*;
            use owasm_abi_derive::eth_abi;

            #[eth_abi(#contract_endpoint, #contract_client)]
            trait #trait_name #methods_group

            pub trait #contract_interface #methods_group

            pub struct #contract_struct<T: #contract_interface>(pub T);

            impl<T: #contract_interface> #contract_struct<T> {
                pub fn deploy(self) {
                    #contract_endpoint::new(self).dispatch_ctor(&owasm_ethereum::input());
                }
                pub fn call(self) {
                    owasm_ethereum::ret(&contract_endpoint::new(self).dispatch(&owasm_ethereum::input()));
                }
            }
        };

        println!("{}", trait_stream.to_string());
    }
}
