extern crate proc_macro;
use proc_macro::TokenStream;

/// Marker macro that does not emit any tokens. Intended for consumption by a separate RAWR binary
/// built on top of Tree-Sitter.
// TODO Ensure that basic properties are populated, or raise a `compile_error!.
#[proc_macro_attribute]
pub fn rawr(_input: TokenStream, _annotated_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
