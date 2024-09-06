use proc_macro::TokenStream;

fn impl_serde_json_roundtrip(name: &syn::Ident) -> TokenStream {
    let gen = quote::quote!(
        impl ::tskit::metadata::MetadataRoundtrip for #name {
            fn encode(&self) -> Result<Vec<u8>, ::tskit::metadata::MetadataError> {
                match ::serde_json::to_string(self) {
                    Ok(x) => Ok(x.as_bytes().to_vec()),
                    Err(e) => {
                        Err(::tskit::metadata::MetadataError::RoundtripError { value: Box::new(e) })
                    }
                }
            }

            fn decode(md: &[u8]) -> Result<Self, ::tskit::metadata::MetadataError> {
                let value: Result<Self, ::serde_json::Error> = ::serde_json::from_slice(md);
                match value {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        Err(::tskit::metadata::MetadataError::RoundtripError { value: Box::new(e) })
                    }
                }
            }
        }
    );
    gen.into()
}

fn impl_serde_bincode_roundtrip(name: &syn::Ident) -> TokenStream {
    let gen = quote::quote!(
        impl ::tskit::metadata::MetadataRoundtrip for #name {
            fn encode(&self) -> Result<Vec<u8>, ::tskit::metadata::MetadataError> {
                match ::bincode::serialize(&self) {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        Err(::tskit::metadata::MetadataError::RoundtripError { value: Box::new(e) })
                    }
                }
            }
            fn decode(md: &[u8]) -> Result<Self, ::tskit::metadata::MetadataError> {
                match ::bincode::deserialize(md) {
                    Ok(x) => Ok(x),
                    Err(e) => {
                        Err(::tskit::metadata::MetadataError::RoundtripError { value: Box::new(e) })
                    }
                }
            }
        }
    );
    gen.into()
}

fn impl_metadata_roundtrip_macro(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &ast.ident;
    let attrs = &ast.attrs;

    for attr in attrs.iter() {
        if attr.path.is_ident("serializer") {
            let lit: syn::LitStr = attr.parse_args().unwrap();
            let serializer = lit.value();

            if &serializer == "serde_json" {
                return Ok(impl_serde_json_roundtrip(name));
            } else if &serializer == "bincode" {
                return Ok(impl_serde_bincode_roundtrip(name));
            } else {
                proc_macro_error2::abort!(serializer, "is not a supported protocol.");
            }
        } else {
            proc_macro_error2::abort!(attr.path, "is not a supported attribute.");
        }
    }

    proc_macro_error2::abort_call_site!("missing [serializer(...)] attribute")
}

macro_rules! make_derive_metadata_tag {
    ($function: ident, $metadatatag: ident) => {
        #[proc_macro_error2::proc_macro_error]
        #[proc_macro_derive($metadatatag, attributes(serializer))]
        /// Register a type as metadata.
        pub fn $function(input: TokenStream) -> TokenStream {
            let ast: syn::DeriveInput = match syn::parse(input) {
                Ok(ast) => ast,
                Err(err) => proc_macro_error2::abort_call_site!(err),
            };
            let mut roundtrip = impl_metadata_roundtrip_macro(&ast).unwrap();
            let name = &ast.ident;
            let gen: proc_macro::TokenStream = quote::quote!(
                impl ::tskit::metadata::$metadatatag for #name {}
            )
            .into();
            roundtrip.extend(gen);
            roundtrip
        }
    };
}

make_derive_metadata_tag!(individual_metadata_derive, IndividualMetadata);
make_derive_metadata_tag!(mutation_metadata_derive, MutationMetadata);
make_derive_metadata_tag!(site_metadata_derive, SiteMetadata);
make_derive_metadata_tag!(population_metadata_derive, PopulationMetadata);
make_derive_metadata_tag!(node_metadata_derive, NodeMetadata);
make_derive_metadata_tag!(edge_metadata_derive, EdgeMetadata);
make_derive_metadata_tag!(migration_metadata_derive, MigrationMetadata);
