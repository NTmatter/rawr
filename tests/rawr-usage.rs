// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]

use rawr_attribute::rawr;
// use rawr_addributes::rawr;
// use rawr::rawr_body;

fn main() {
    println!("Testing!")
}

#[rawr(
    src = "x::y",
    rev = "abc123def",
    implemented = true,
    notes = "It took a while to implement this, and I think it's done. Does it print the right number?"
)]
fn foo(bar: usize) -> Result<(), !> {
    println!("There are {bar} lights!");
    Ok(())
}

#[rawr(a = 1, b = "abc")]
#[rawr(not_relevant = "skipped")]
#[rawr(c = 2, d = "def")]
#[derive(Debug)]
struct S {
    a: Option<String>,
}

#[rawr(e = 3, f = "ghi")]
const FOO: u8 = 1;

#[rawr(on_enum = true, g = 4, h = "jkl")]
enum Foo {
    a,
    #[rawr(on_variant = true)]
    b,
}

fn bar() {
    // Comment?
    /* Comment! */
    // I need an alternate version that runs as a declarative macro
    // for use inside function bodies.
    // rawr!(on_statement = true);
    let x = 1;
}
