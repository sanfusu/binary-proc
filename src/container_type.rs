use syn::{parse::Parse, punctuated::Punctuated, Token, TypePath};

pub(crate) struct ContainerType {
    pub(crate) types: Vec<TypePath>,
}

impl Parse for ContainerType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let puncated_types = Punctuated::<TypePath, Token!(,)>::parse_separated_nonempty(input)?;
        let mut types = Vec::new();
        puncated_types.into_iter().for_each(|x| types.push(x));
        Ok(ContainerType { types })
    }
}
