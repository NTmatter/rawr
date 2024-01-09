// SPDX-License-Identifier: MIT

// NOTE: The annotations are MIT-licensed for maximum compatibility with downstream codebases.
// All of the heavy lifting occurs in the binaries, and they are Apache-2.0 licensed.
// TODO Split binaries and lib into separate crates for clearer licensing and usage.
extern crate proc_macro;
use proc_macro::TokenStream;

/// Marker macro that does not emit any tokens. Intended for consumption by a separate RAWR binary
/// built on top of Tree-Sitter.
// TODO Ensure that basic properties are populated, or raise a `compile_error!.
#[proc_macro_attribute]
pub fn rawr(_input: TokenStream, _annotated_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn rawr2(_input: TokenStream, _annotated_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
