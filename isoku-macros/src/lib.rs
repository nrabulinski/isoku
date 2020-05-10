extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    Fields, Ident, ItemStruct, Token, Type,
};

struct Decoder {
    name: Ident,
    args: Vec<Ident>,
}

impl Parse for Decoder {
    fn parse(input: ParseStream) -> Result<Decoder> {
        let name: Ident = input.parse()?;
        let args = match input.parse::<Token![:]>() {
            Ok(_) => Punctuated::<Ident, Token![,]>::parse_terminated(input)?
                .into_iter()
                .collect(),
            Err(_) => Vec::new(),
        };
        Ok(Decoder { name, args })
    }
}

enum TType {
    Owned(TokenStream2),
    Borrowed(TokenStream2),
}

#[proc_macro_attribute]
pub fn event_data(_args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as ItemStruct);
    let redefined = redefine_struct(parsed);

    TokenStream::from(redefined)
}

fn type_to_tt(ty: &Type) -> TType {
    match ty {
        Type::Reference(t) => TType::Borrowed(t.elem.to_token_stream()),
        Type::Path(t) => TType::Owned({
            // let ty = f.ty.clone();
            let seg = t.path.segments.first().unwrap();
            use syn::PathArguments as PA;
            match &seg.arguments {
                PA::None => ty.to_token_stream(),
                PA::AngleBracketed(arg) => {
                    use syn::GenericArgument as GA;
                    let args = arg
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            GA::Type(a) => Some(type_to_tt(a)),
                            _ => None,
                        })
                        .map(|a| match a {
                            TType::Borrowed(stream) => quote!(&'a #stream),
                            TType::Owned(stream) => stream,
                        });
                    let t = &seg.ident;
                    quote!(#t < #(#args),* > )
                }
                _ => unreachable!(),
            }
            // dbg!(&f.ty).to_token_stream()
        }),
        Type::Array(arr) => TType::Owned(arr.to_token_stream()),
        e => {
            println!("----------------\n{:#?}\n-----------------", e);
            unreachable!()
        }
    }
}

fn redefine_struct(input: ItemStruct) -> TokenStream2 {
    let fields = match input.fields {
        Fields::Named(a) => a
            .named
            .iter()
            .map(|f| {
                (
                    f.ident.as_ref().unwrap().clone(),
                    type_to_tt(&f.ty),
                    f.attrs
                        .iter()
                        .find(|a| a.path.is_ident("decoder"))
                        .map(|a| a.parse_args::<Decoder>().unwrap()),
                )
            })
            .collect::<Vec<(Ident, TType, Option<Decoder>)>>(),
        _ => unimplemented!(),
    };
    let name = &input.ident;
    let attrs = &input.attrs;
    let fields_parsed = fields.iter().map(|(ident, t, _)| {
        let t = match t {
            TType::Owned(t) => t.clone(),
            TType::Borrowed(t) => quote!(&'a #t),
        };
        quote!(#ident : #t)
    });
    let fields_idents = fields.iter().map(|(ident, ..)| ident);
    let fields_impl = fields.iter().map(|(ident, t, f)| {
        let decoder = f.as_ref().map(|i| {
            let name = &i.name;
            let args = i.args.iter().map(|arg| quote!(&#arg));
            quote!(#name(data #(, #args)*))
        });
        let name = match decoder {
            None => match t {
                TType::Owned(_) => quote!(& #ident),
                TType::Borrowed(_) => ident.to_token_stream(),
            },
            Some(_) => ident.to_token_stream(),
        };
        let t = match t {
            TType::Owned(t) | TType::Borrowed(t) => t,
        };
        let decoder = decoder.unwrap_or_else(|| quote!(<#t as OsuEncode>::decode(data)));
        quote! {
            let data = &data[off..];
            let (#name, off) = #decoder?;
        }
    });
    quote! {
        #(#attrs)*
        struct #name <'a> {
            #(#fields_parsed),*
        }

        impl <'a> #name <'a> {
            fn decode(data: &'a [u8]) -> Result<#name, ()> {
                use crate::packets::OsuEncode;
                let off = 0;
                #(#fields_impl)*
                Ok(#name { #(#fields_idents),* })
            }
        }
    }
}
