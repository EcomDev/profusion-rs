use quote::format_ident;
use syn::{Error, Ident, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub(crate) struct ScenarioArguments {
    name: Ident,
    builder_name: Ident,
}

impl Parse for ScenarioArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let builder_name = input
            .parse::<Ident>()
            .map_err(|e| Error::new(e.span(), "missing struct name for scenario"))?;

        let mut name = format_ident!("{builder_name}Scenario");

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            name = input.parse::<Ident>()?;
        }

        if !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "There should be only two attributes",
            ));
        }

        Ok(Self { name, builder_name })
    }
}

impl ScenarioArguments {
    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn builder_name(&self) -> &Ident {
        &self.builder_name
    }
}

#[cfg(test)]
mod test {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn empty_arguments_return_error() {
        let arguments = parse2::<ScenarioArguments>(quote! {});

        assert!(arguments.is_err());
    }

    #[test]
    fn missing_second_struct_name_gets_gets_handled() {
        let arguments = parse2::<ScenarioArguments>(quote! { ItemName, });
        assert!(arguments.is_err());
    }

    #[test]
    fn too_many_arguments_result_in_error() {
        let arguments = parse2::<ScenarioArguments>(quote! { ItemName, ItemNameBuilder, TooMuch });

        assert!(arguments.is_err());
        print!("{}", arguments.unwrap_err().to_string());
    }

    #[test]
    fn creates_args_with_default_builder_suffix() {
        let arguments = parse2::<ScenarioArguments>(quote! { ItemName }).unwrap();
        assert_eq!(
            arguments.builder_name,
            Ident::new("ItemName", arguments.builder_name.span())
        );
        assert_eq!(
            arguments.name,
            Ident::new("ItemNameScenario", arguments.name.span())
        );
    }

    #[test]
    fn creates_args_with_custom_scenario_suffix() {
        let arguments =
            parse2::<ScenarioArguments>(quote! { ItemName, ItemNameCustomScenario }).unwrap();
        assert_eq!(
            arguments.name,
            Ident::new("ItemNameCustomScenario", arguments.name.span())
        );
        assert_eq!(
            arguments.builder_name,
            Ident::new("ItemName", arguments.builder_name.span())
        );
    }
}
