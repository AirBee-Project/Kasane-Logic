//! kasane-logic の derive マクロ。利用者は `kasane_logic` から再公開されたものを使う。

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

/// バリアント数の上限。`ValueSet` の `u64` の 1 ビットに 1 つずつ対応させるため。
const MAX_VARIANTS: usize = 64;

/// フィールドを持たない enum に `BitMask` を実装する。
///
/// バリアントの番号は定義順に `0, 1, 2, …` となり、判別値（`A = 5`）には左右されない。
#[proc_macro_derive(BitMask)]
pub fn derive_bit_mask(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    bit_mask(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn bit_mask(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "BitMask は enum にだけ実装できる",
        ));
    };
    if let Some(variant) = data
        .variants
        .iter()
        .find(|v| !matches!(v.fields, Fields::Unit))
    {
        return Err(Error::new_spanned(
            variant,
            "BitMask のバリアントはフィールドを持てない",
        ));
    }
    if data.variants.len() > MAX_VARIANTS {
        return Err(Error::new_spanned(
            &input.ident,
            format!("BitMask のバリアントは {MAX_VARIANTS} 個以下にすること"),
        ));
    }

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let count = data.variants.len() as u32;
    let arms = data.variants.iter().zip(0u32..).map(|(variant, index)| {
        let variant = &variant.ident;
        quote! { Self::#variant => #index }
    });

    Ok(quote! {
        impl #impl_generics ::kasane_logic::spatial_id::collection::flex_tree_2::BitMask for #name #type_generics #where_clause {
            const COUNT: u32 = #count;

            #[inline]
            fn index(self) -> u32 {
                match self {
                    #(#arms,)*
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn error_of(input: DeriveInput) -> String {
        bit_mask(&input).unwrap_err().to_string()
    }

    #[test]
    fn numbers_variants_in_definition_order() {
        let output = bit_mask(&parse_quote! {
            enum Color { Red, Green = 10, Blue }
        })
        .unwrap()
        .to_string();
        assert!(output.contains("const COUNT : u32 = 3u32"));
        assert!(output.contains("Self :: Green => 1u32"));
    }

    #[test]
    fn rejects_struct() {
        let error = error_of(parse_quote! { struct Color; });
        assert!(error.contains("enum にだけ"));
    }

    #[test]
    fn rejects_variant_with_fields() {
        let error = error_of(parse_quote! { enum Shape { Point, Circle(u32) } });
        assert!(error.contains("フィールドを持てない"));
    }

    #[test]
    fn rejects_more_than_64_variants() {
        let variants = (0..=MAX_VARIANTS).map(|i| quote::format_ident!("V{i}"));
        let error = error_of(parse_quote! { enum Many { #(#variants),* } });
        assert!(error.contains("64 個以下"));
    }
}
