use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    AssocType,
    FnArg,
    GenericArgument, Ident, ItemFn, parse::{Parse, ParseStream}, Pat, Path, PathArguments, PathSegment, PatType,
    ReturnType, Signature, spanned::Spanned, TraitBound, Type, TypeParamBound, TypePath,
};

use crate::scenario_arguments::ScenarioArguments;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct LoadTestFunction {
    function: ItemFn,
    state: Vec<(Ident, Path)>,
    metric_type: Type,
}

impl Parse for LoadTestFunction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let function = input.parse::<ItemFn>()?;

        if function.sig.asyncness.is_none() {
            return Err(syn::Error::new(
                function.sig.span(),
                "scenario must be async function",
            ));
        }

        let (state, metric_type) = process_function_input(&function.sig)?;

        validate_return_type(&function.sig.output)?;

        Ok(Self {
            function,
            state,
            metric_type,
        })
    }
}

impl LoadTestFunction {
    fn builder_definition(&self, name: &Ident) -> TokenStream {
        quote! {
            struct #name;
        }
    }

    fn scenario_definition(&self, name: &Ident) -> TokenStream {
        let state: Vec<_> = self
            .state
            .iter()
            .map(|(ident, path)| {
                quote! {
                    #ident: #path,
                }
            })
            .collect();

        if !self.state.is_empty() {
            return quote! {
                struct #name { #( #state )* }
            };
        }

        quote! { struct #name; }
    }

    fn scenario_default(&self, name: &Ident) -> TokenStream {
        let state: Vec<_> = self
            .state
            .iter()
            .map(|(ident, _)| {
                quote! {
                    #ident: Default::default(),
                }
            })
            .collect();

        if !self.state.is_empty() {
            return quote! {
                #name { #( #state )* }
            };
        }

        quote! {
            #name
        }
    }

    fn builder_impl(&self, name: &Ident, builder_name: &Ident) -> TokenStream {
        let build_creator = self.scenario_default(name);
        let metric = &self.metric_type;

        quote! {
            impl profusion::prelude::ScenarioBuilder<#metric> for #builder_name {
                type Scenario = #name;

                fn build(&self) -> Self::Scenario {
                    #build_creator
                }
            }
        }
    }

    fn scenario_impl(&self, name: &Ident) -> TokenStream {
        let metric = &self.metric_type;
        let state: Vec<_> = self
            .state
            .iter()
            .map(|(ident, _)| {
                quote! {
                    &mut self.#ident
                }
            })
            .collect();

        let function_name = &self.function.sig.ident;

        quote! {
            impl profusion::prelude::Scenario<#metric> for #name {
                async fn execute(
                    &mut self,
                    aggregate: &mut MetricMeasurer< impl MetricAggregate<Metric = #metric> >
                ) -> Result<(), MetricRecordError> {
                    #function_name(aggregate #(, #state )*).await
                }
            }
        }
    }

    pub(crate) fn generate(&self, arguments: &ScenarioArguments) -> TokenStream {
        let builder_definition = self.builder_definition(arguments.builder_name());
        let builder_impl = self.builder_impl(arguments.name(), arguments.builder_name());
        let scenario_definition = self.scenario_definition(arguments.name());
        let scenario_impl = self.scenario_impl(arguments.name());
        let function_definition = &self.function;

        quote! {
            #builder_definition
            #scenario_definition
            #builder_impl
            #scenario_impl
            #function_definition
        }
    }
}

fn process_function_input(signature: &Signature) -> Result<(Vec<(Ident, Path)>, Type), syn::Error> {
    let metric_type = match signature.inputs.first() {
        Some(FnArg::Typed(path))
            if is_of_type(&path.ty, "MetricMeasurer") && is_mutable_reference(&path.ty) =>
        {
            extract_last_path_segment(&path.ty)
        }
        _ => None,
    };

    let metric_type = match metric_type {
        Some(PathSegment {
            arguments: PathArguments::AngleBracketed(arguments),
            ..
        }) => match arguments.args.first().map(extract_measurer_aggregate_metric) {
            Some(Some(Ok(path))) => path,
            Some(Some(Err(err))) => return Err(err),
            _ => {
                return Err(syn::Error::new(
                    signature.inputs.span(),
                    "missing required `impl MetricAggregate<Metric=T>` in `MetricMeasurer`",
                ))
            }
        },
        Some(PathSegment {
            arguments: PathArguments::None,
            ..
        }) => {
            return Err(syn::Error::new(
                signature.inputs.span(),
                "missing required `impl MetricAggregate<Metric=T>` in `MetricMeasurer`",
            ))
        }
        _ => return Err(syn::Error::new(
            signature.inputs.span(),
            "missing required `&mut profusion::prelude::MetricMeasurer` as first function argument",
        )),
    };

    let mut types = Vec::new();

    for item in signature.inputs.iter().skip(1) {
        match item {
            FnArg::Typed(PatType { ty, pat, .. }) => {
                match (
                    extract_type_path(ty),
                    *(pat.clone()),
                    is_mutable_reference(ty),
                ) {
                    (Some(type_name), Pat::Ident(ident), true) => {
                        types.push((ident.ident, type_name.path.clone()))
                    }
                    _ => return Err(syn::Error::new(
                        item.span(),
                        "state for load tests should be a &mut type that implements Default trait",
                    )),
                }
            }
            _ => continue,
        }
    }

    Ok((types, metric_type))
}

fn is_of_type(item_type: &Type, expected: &str) -> bool {
    extract_type_path(item_type).map_or(false, |path| is_path_of_type(&path.path, expected))
}

fn is_path_of_type(path: &Path, expected: &str) -> bool {
    matches!(
        path.segments.last(),
        Some(path) if path.ident.eq(&Ident::new(expected, path.ident.span()))
    )
}

fn extract_type_path(item_type: &Type) -> Option<&TypePath> {
    match item_type {
        Type::Path(item) => Some(item),
        Type::Reference(reference) => extract_type_path(&reference.elem),
        _ => None,
    }
}

fn extract_measurer_aggregate_metric(
    argument: &GenericArgument,
) -> Option<Result<Type, syn::Error>> {
    match argument {
        GenericArgument::Type(Type::ImplTrait(trait_impl)) => match trait_impl.bounds.first() {
            Some(TypeParamBound::Trait(TraitBound { path, .. }))
                if is_path_of_type(path, "MetricAggregate") =>
            {
                match path.segments.last()? {
                    PathSegment {
                        arguments: PathArguments::AngleBracketed(args),
                        ..
                    } => match args.args.first()? {
                        GenericArgument::AssocType(AssocType { ident, ty, .. })
                            if ident.eq(&Ident::new("Metric", ident.span())) =>
                        {
                            Some(Ok(ty.clone()))
                        }
                        _ => Some(Err(syn::Error::new(
                            args.args.first()?.span(),
                            "missing required `T` of in `impl MetricAggregate<Metric=T>`",
                        ))),
                    },
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

fn extract_last_path_segment(item_type: &Type) -> Option<&PathSegment> {
    match extract_type_path(item_type)?.path.segments.last() {
        Some(path) => Some(path),
        _ => None,
    }
}

fn is_mutable_reference(item_type: &Type) -> bool {
    matches!(item_type, Type::Reference(reference) if reference.mutability.is_some())
}

fn validate_return_type(result: &ReturnType) -> Result<(), syn::Error> {
    if let ReturnType::Type(_, return_type) = result {
        if is_of_type(return_type, "Result") {
            return Ok(());
        }
    };

    Err(syn::Error::new(
        result.span(),
        "invalid return type, it should be std::result::Result with `profusion::prelude::MetricRecordError` as Error",
    ))
}

#[cfg(test)]
mod load_test_function_tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn returns_error_on_non_async_function() {
        let stream = quote! {
            fn load_test(measurer: &mut MetricMeasurer<impl MetricAggregate<Metric = TestMetric>>)
            -> Result<(), MetricRecordError> {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_on_missing_measurer_argument() {
        let stream = quote! {
            async fn load_test() -> Result<(), MetricRecordError> {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_on_empty_metric() {
        let stream = quote! {
            async fn load_test(
                measurer: &mut MetricMeasurer<impl MetricAggregate>
            ) -> Result<(), MetricRecordError> {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_not_mut_arguments() {
        let stream = quote! {
            async fn load_test<A>(
                measurer: &mut profusion::prelude::MetricMeasurer<A>,
                state: &mut State,
                state_two: State2
            ) -> Result<(), MetricRecordError> where A: Instance {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_when_not_result_return_type() {
        let stream = quote! {
            async fn load_test<A>(
                measurer: &mut profusion::prelude::MetricMeasurer<A>,
                state: &mut State,
                state_two: State2
            ) where A: Instance {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_when_wrong_result_error_returned() {
        let stream = quote! {
            async fn load_test<A>(
                measurer: &mut profusion::prelude::MetricMeasurer<A>,
                state: &mut State, state_two: State2
            ) -> Result<(), Test> where A: Instance {

            }
        };

        let result = parse2::<LoadTestFunction>(stream);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod process_function_arguments_tests {
    use quote::quote;
    use quote::ToTokens;
    use syn::{parse2, Signature};

    use super::*;

    #[test]
    fn first_argument_of_function_is_correct() {
        let (state, metric_type) = process_function_input(
            &parse2::<Signature>(
                quote! { async fn load_test(measurer: &mut MetricMeasurer<impl MetricAggregate<Metric=Test>>) },
            )
            .unwrap(),
        )
        .unwrap();

        let ident = metric_type.to_token_stream();

        assert_eq!(state.len(), 0);
        assert_eq!(ident.to_string(), quote! { Test }.to_string());
    }

    #[test]
    fn error_when_first_argument_of_function_is_missing() {
        let result = process_function_input(
            &parse2::<Signature>(quote! { async fn load_test<A>() }).unwrap(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn error_when_first_argument_of_function_is_not_mutable_ref() {
        let result = process_function_input(
            &parse2::<Signature>(quote! { async fn load_test<A>(measurer: MetricMeasurer<A>) })
                .unwrap(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn error_when_first_argument_function_is_not_mutable() {
        let result = process_function_input(
            &parse2::<Signature>(quote! { async fn load_test<A>(measurer: &mut State) }).unwrap(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn error_when_other_arguments_are_not_mutable_references() {
        let result = process_function_input(
            &parse2::<Signature>(quote! {
                async fn load_test<A>(
                    measurer: &mut MetricMeasurer<A>,
                    state_one: &mut item_crate::StateOne,
                    state_two: item_crate::StateTwo,
                )
            })
            .unwrap(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn returns_all_state_arguments() {
        let (result, _metric_type) = process_function_input(
            &parse2::<Signature>(quote! {
                async fn load_test(
                    measurer: &mut MetricMeasurer<impl MetricAggregate<Metric=TestTwo>>,
                    state_one: &mut item_crate::StateOne,
                    state_two: &mut item_crate::StateTwo,
                    state_three: &mut StateThree
                )
            })
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result[0].0, Ident::new("state_one", result[0].0.span()));
        assert_eq!(result[1].0, Ident::new("state_two", result[0].0.span()));
        assert_eq!(result[2].0, Ident::new("state_three", result[0].0.span()));
    }
}

#[cfg(test)]
mod validate_result_type_tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn accepts_full_metric_error_in_result() {
        let return_type = validate_return_type(
            &parse2::<ReturnType>(quote! { -> Result<(), profusion::prelude::MetricRecordError> })
                .unwrap(),
        );

        assert!(return_type.is_ok());
    }

    #[test]
    fn accepts_simplified_metric_error_in_result() {
        let return_type = validate_return_type(
            &parse2::<ReturnType>(quote! { -> Result<(), MetricRecordError> }).unwrap(),
        );

        assert!(return_type.is_ok());
    }

    #[test]
    fn accepts_any_error_result_that_converts_to_metric_error() {
        let return_type = validate_return_type(
            &parse2::<ReturnType>(quote! { -> Result<(), std::io::Error> }).unwrap(),
        );

        assert!(return_type.is_ok());
    }

    #[test]
    fn does_not_accept_wrong_return_type() {
        let return_type = validate_return_type(&parse2::<ReturnType>(quote! { -> () }).unwrap());

        assert!(return_type.is_err());
    }

    #[test]
    fn does_not_accept_empty_return_type() {
        let return_type = validate_return_type(&parse2::<ReturnType>(quote! {}).unwrap());

        assert!(return_type.is_err());
    }
}
