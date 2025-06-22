// SPDX-License-Identifier: Apache-2.0

//! Example usage of `#[rawr]` annotations.

#![allow(unused)]

use rawr::{rawr, rawr_fn, Rawr};

// If the grammar provides a source_file match, it can be leveraged to watch an
// entire file for changes.
#[rawr(
    file = "examples/test.cpp",
    kind = "source_file",
    rev = "abc123",
    note = "Match a whole file to watch for any changes"
)]
#[rawr(
    file = "images/cat.jpg",
    kind = "source_file",
    rev = "0001",
    ignore = true,
    note = "This is a CLI app, stick to emoji 😸"
)]
// Must be attached to a throwaway item, as inner attributes are not yet stable.
// See rust-lang/rust#54726 for details.
const _RAWR: () = ();

// Upstream items can also be ignored if they are not relevant to the implementation.

fn main() {
    println!("Testing!")
}

// Watch a function body for changes. Modifications to the args may require
// changing the watched ident to match.
#[rawr(
    upstream = "the-original",
    file = "src/counter.c",
    kind = "function",
    ident = "light_counter(int* lights)",
    rev = "bcd234",
    state = "WIP",
    action = "Testing needed",
    notes = "It took a while to implement this, and I think it's \"done\" -- Does it print the right number?"
)]
fn foo(bar: usize) -> Result<(), String> {
    println!("There are {bar} lights!");

    Ok(())
}

// It works for structs!
#[rawr(file = "src/example.c", kind = "struct", ident = "S", rev = "abc123")]
#[derive(Debug)]
struct S {
    a: Option<String>,
}

// It works for structs and enums
#[rawr(file = "src/example.c", kind = "const", ident = "S", rev = "abc123")]
const FOO: u8 = 1;

#[rawr(
    file = "src/example.c",
    kind = "enum",
    ident = "LETTERS",
    rev = "abc123"
)]
enum Letters {
    A,
    B,
    C,
}

// Upstream constants can be collected into enums that are monitored for changes.
#[derive(Rawr)]
#[repr(u16)]
enum Foo {
    #[rawr(
        file = "include/constants.h",
        kind = "const",
        ident = "COPYRIGHT_YEAR",
        rev = "abc123"
    )]
    CopyrightYear = 1997,

    #[rawr(
        file = "include/constants.h",
        kind = "const",
        ident = "COPYRIGHT_YEAR",
        rev = "def234",
        notes = "There were a lot of edits."
    )]
    PublishingYear = 2012,

    #[rawr(
        file = "include/constants.h",
        kind = "const",
        ident = "RELEASE_VERSION",
        rev = "bcd456"
    )]
    Version = 2,
}

// Multiple annotations are supported to allow dependencies on multiple
// upstream items, even from different branches or codebases.
#[rawr(
    upstream = "the-original",
    file = "src/foo.c",
    kind = "function",
    ident = "bar(int drinks)",
    rev = "abc123",
    state = "DONE"
)]
#[rawr(
    upstream = "the-original",
    file = "include/foo.h",
    kind = "function_declarator",
    ident = "bar(int)",
    rev = "def000",
    notes = "Header file"
)]
fn bar() {
    rawr_fn!(
        file = "numbers.h",
        kind = "constant",
        ident = "X",
        rev = "123"
    );
    let x = 1;
    let y = Letters::A;
}
