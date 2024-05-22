use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::load_test_function::LoadTestFunction;
use crate::scenario_arguments::ScenarioArguments;

pub(crate) struct Scenario {
    scenario_arguments: ScenarioArguments,
    function: LoadTestFunction,
}

impl Scenario {
    pub(crate) fn new(scenario_arguments: ScenarioArguments, function: LoadTestFunction) -> Self {
        Self {
            scenario_arguments,
            function,
        }
    }
}

impl ToTokens for Scenario {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.function.generate(&self.scenario_arguments).to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn generates_stateless_code() {
        let scenario = Scenario::new(
            parse2(quote! { MyLoadTest }).unwrap(),
            parse2(
                quote! { async fn test_function(report: &mut MetricMeasurer<impl MetricAggregate<Metric=&'static str>>) -> Result<(), Error> {}},
            )
            .unwrap(),
        );

        assert_eq!(
            scenario.to_token_stream().to_string(),
            quote! {
                struct MyLoadTest;
                struct MyLoadTestScenario;

                impl profusion::prelude::ScenarioBuilder<&'static str> for MyLoadTest {
                    type Scenario = MyLoadTestScenario;
                    fn build(& self) -> Self::Scenario {
                        MyLoadTestScenario
                    }
                }

                impl profusion::prelude::Scenario<&'static str> for MyLoadTestScenario {
                    async fn execute(
                        &mut self,
                        aggregate: &mut MetricMeasurer<impl MetricAggregate<Metric=& 'static str> >
                    ) -> Result<(), MetricRecordError> {
                        test_function(aggregate).await
                    }
                }

                async fn test_function(report: &mut MetricMeasurer<impl MetricAggregate<Metric=&'static str> >)
                    -> Result<(), Error> {

                }
            }
            .to_string()
        );
    }

    #[test]
    fn generates_stateful_code() {
        let scenario = Scenario::new(
            parse2(quote! { MyLoadTest }).unwrap(),
            parse2(
                quote! {
                    async fn test_function(report: &mut MetricMeasurer<impl MetricAggregate<Metric=&'static str>>, state_one: &mut StateOne, state_two: &mut StateTwo,) -> Result<(), Error> {}},
            )
                .unwrap(),
        );

        assert_eq!(
            scenario.to_token_stream().to_string(),
            quote! {
                struct MyLoadTest;
                struct MyLoadTestScenario {
                     state_one: StateOne,
                     state_two: StateTwo,
                }

                impl profusion::prelude::ScenarioBuilder<&'static str> for MyLoadTest {
                    type Scenario = MyLoadTestScenario;
                    fn build(& self) -> Self::Scenario {
                        MyLoadTestScenario {
                            state_one: Default::default(),
                            state_two: Default::default(),
                        }
                    }
                }

                impl profusion::prelude::Scenario<&'static str> for MyLoadTestScenario {
                    async fn execute(
                        &mut self,
                        aggregate: &mut MetricMeasurer<impl MetricAggregate<Metric=& 'static str> >
                    ) -> Result<(), MetricRecordError> {
                        test_function(aggregate, &mut self.state_one, &mut self.state_two).await
                    }
                }

                async fn test_function(report: &mut MetricMeasurer<impl MetricAggregate<Metric=&'static str> >, state_one: &mut StateOne, state_two: &mut StateTwo, )
                    -> Result<(), Error> {

                }
            }
                .to_string()
        );
    }
}
