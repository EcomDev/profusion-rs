use proc_macro::TokenStream;

use quote::ToTokens;
use syn::parse_macro_input;

use crate::load_test_function::LoadTestFunction;
use crate::scenario::Scenario;
use crate::scenario_arguments::ScenarioArguments;

mod load_test_function;
mod scenario;
mod scenario_arguments;

#[proc_macro_attribute]
pub fn scenario(args: TokenStream, item: TokenStream) -> TokenStream {
    let scenario = Scenario::new(
        parse_macro_input!(args as ScenarioArguments),
        parse_macro_input!(item as LoadTestFunction),
    );

    scenario.to_token_stream().into()
}
