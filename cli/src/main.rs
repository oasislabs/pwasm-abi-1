#![recursion_limit = "192"]

extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate clap;
extern crate owasm_abi_cli;
extern crate syn;

use std::fs::{self, File};
use std::io::Read;

use clap::{App, Arg};
use owasm_abi_cli::rustfmt;
use proc_macro2::{Group, Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Colon2, Comma, Pound};
use syn::AttrStyle::Outer;
use syn::{Attribute, FnArg, Ident, MethodSig, Pat, Path, PathArguments, PathSegment};

macro_rules! format_ident {
    ($ident:expr, $fstr:expr) => {
        syn::Ident::new(&format!($fstr, $ident), $ident.span())
    };
}

fn main() {
    let matches = App::new("owasm-derive")
        .arg(
            Arg::with_name("code")
                .index(1)
                .required(true)
                .help("Path to contract code"),
        )
        .get_matches();
    let mut file = File::open(matches.value_of("code").unwrap()).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

    let ast = syn::parse_file(&code).unwrap();

    let mut punctuated: Punctuated<PathSegment, Colon2> = Punctuated::new();
    punctuated.push(PathSegment {
        ident: Ident::new("owasm_abi_derive", Span::call_site()),
        arguments: PathArguments::None,
    });
    punctuated.push(PathSegment {
        ident: Ident::new("contract", Span::call_site()),
        arguments: PathArguments::None,
    });

    let contract_attribute = Attribute {
        pound_token: Pound::default(),
        style: Outer,
        bracket_token: Bracket::default(),
        path: Path {
            leading_colon: None,
            segments: punctuated,
        },
        tts: TokenStream::new(),
    };

    let traits_to_method_sigs: Vec<(&Ident, Vec<(TokenStream, &MethodSig)>)> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Trait(m) => {
                if m.attrs.contains(&contract_attribute) {
                    let trait_name = &m.ident;

                    let method_sigs = m
                        .items
                        .iter()
                        .filter_map(|item| match item {
                            syn::TraitItem::Method(m) => {
                                let msig = &m.sig;
                                let mattrs = &m.attrs;
                                Some((
                                    quote! {
                                        #(#mattrs)*
                                        #msig;
                                    },
                                    msig,
                                ))
                            }
                            _ => None,
                        })
                        .collect();
                    return Some((trait_name, method_sigs));
                }
                None
            }
            _ => None,
        })
        .collect();

    for (trait_name, method_sigs) in traits_to_method_sigs {
        let contract_endpoint = format_ident!(trait_name, "{}Endpoint");
        let contract_client = format_ident!(trait_name, "{}Client");
        let contract_struct = format_ident!(trait_name, "{}Inst");
        let contract_interface = format_ident!(trait_name, "{}Interface");

        let mut methods_stream = quote!();
        let mut contract_impl_stream = quote!();
        for (method_quote, method_sig) in method_sigs {
            methods_stream.extend(method_quote);

            let method_ident = &method_sig.ident;
            let mut inputs_iter = method_sig.decl.inputs.iter();
            let self_ref_check = inputs_iter.next();
            let self_ref_error = format!(
                "ABI function `{}` must have `&mut self` as its first argument.",
                method_ident.to_string()
            );
            match self_ref_check {
                Some(syn::FnArg::SelfRef(ref selfref)) => {
                    if selfref.mutability.is_none() {
                        panic!(self_ref_error)
                    }
                }
                _ => panic!(self_ref_error),
            }

            let mut arguments_list: Punctuated<Ident, Comma> = Punctuated::new();
            for input in inputs_iter {
                match input {
                    FnArg::Captured(arg_captured) => match &arg_captured.pat {
                        Pat::Ident(pat_ident) => {
                            arguments_list.push(pat_ident.ident.clone());
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }
            if arguments_list.is_empty() {
                contract_impl_stream.extend(quote! {
                    #method_sig {
                        self . 0 . #method_ident()
                    }
                });
            } else {
                contract_impl_stream.extend(quote! {
                    #method_sig {
                        self . 0 . #method_ident(#arguments_list)
                    }
                });
            }
        }

        let contract_impl_group = Group::new(proc_macro2::Delimiter::Brace, contract_impl_stream);
        let methods_group = Group::new(proc_macro2::Delimiter::Brace, methods_stream);
        let trait_stream = quote! {
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

            impl<T: #contract_interface> #trait_name for #contract_struct<T> #contract_impl_group
        };

        let (output, err) = rustfmt(trait_stream.to_string());
        if !err.is_empty() {
            panic!("Failed to format the code: {}", err);
        }
        fs::create_dir_all("generated/src").unwrap();
        fs::write("generated/src/lib.rs", output).unwrap();
    }
}
