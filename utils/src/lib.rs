#[macro_use]
extern crate quote;

macro_rules! format_ident {
    ($ident:expr, $fstr:expr) => {
        syn::Ident::new(&format!($fstr, $ident), $ident.span())
    };
}

pub struct Contract {
    pub trait_name: proc_macro2::Ident,
    pub endpoint_name: proc_macro2::Ident,
    pub client_name: proc_macro2::Ident,
    pub struct_name: proc_macro2::Ident,
    pub method_sigs: Vec<proc_macro2::TokenStream>,
    pub method_impls: Vec<proc_macro2::TokenStream>,
}

impl Contract {
    pub fn new(contract_trait: &syn::ItemTrait) -> Self {
        let (method_sigs, method_impls) = split_contract_trait(&contract_trait);
        Contract {
            trait_name: format_ident!(contract_trait.ident, "{}"),
            endpoint_name: format_ident!(contract_trait.ident, "{}Endpoint"),
            client_name: format_ident!(contract_trait.ident, "{}Client"),
            struct_name: format_ident!(contract_trait.ident, "{}Inst"),
            method_sigs: method_sigs,
            method_impls: method_impls,
        }
    }
}

fn split_contract_trait(
    contract_trait: &syn::ItemTrait,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    contract_trait
        .items
        .iter()
        .filter_map(|itm| match itm {
            syn::TraitItem::Method(m) => {
                let msig = &m.sig;
                let bad_self_ref = format!(
                    "ABI function `{}` must have `&mut self` as its first argument.",
                    msig.ident.to_string()
                );
                match msig.decl.inputs[0] {
                    syn::FnArg::SelfRef(ref selfref) => {
                        if selfref.mutability.is_none() {
                            panic!(bad_self_ref)
                        }
                    }
                    _ => panic!(bad_self_ref),
                }

                let mattrs = &m.attrs;
                let sig = quote! {
                  #(#mattrs)*
                  #msig;
                };

                let body = match m.default {
                    Some(ref mbody) => {
                        quote! { #msig { #mbody } }
                    }
                    None => quote! {},
                };

                Some((sig, body))
            }
            _ => None,
        })
        .unzip()
}
