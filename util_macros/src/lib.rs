use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(EnumFromArgsDeserialize)]
pub fn command_deserialize_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = if let Data::Enum(data) = &input.data {
        &data.variants
    } else {
        panic!("EnumFromArgsDeserialize can only be derived for enums");
    };

    let match_arms: Vec<_> = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let command_name = variant_name.to_string().to_lowercase();

        match &variant.fields {
            syn::Fields::Unit => {
                quote! {
                    #command_name => {
                        if parts.next().is_some() {
                            return Err(format!("{} command takes no arguments", #command_name));
                        }
                        Ok(Self::#variant_name)
                    }
                }
            }
            syn::Fields::Named(fields) => {
                let field_idents: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_count = field_idents.len();
                let field_parsers: Vec<_> = field_idents.iter().enumerate().map(|(i, ident)| {
                    quote! {
                        let #ident = parts.next()
                            .ok_or(format!("{} requires {} arguments", #command_name, #field_count))?
                            .parse()
                            .map_err(|e| format!("Invalid argument {}: {}", #i, e))?;
                    }
                }).collect();

                quote! {
                    #command_name => {
                        #(#field_parsers)*

                        if parts.next().is_some() {
                            return Err(format!("{} takes exactly {} arguments", #command_name, #field_count));
                        }

                        Ok(Self::#variant_name {
                            #(#field_idents),*
                        })
                    }
                }
            }
            syn::Fields::Unnamed(_) => {
                panic!("Tuple variants are not supported");
            }
        }
    }).collect();

    let expanded = quote! {
        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut parts = s.split_whitespace();
                let command = parts.next().ok_or("Empty string")?.to_lowercase();

                match command.as_str() {
                    #(#match_arms)*
                    _ => Err(format!("Unknown command: {}", command)),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                Self::from_str(&s).map_err(serde::de::Error::custom)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
