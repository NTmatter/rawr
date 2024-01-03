use rawr::rawr;
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
#[derive(Debug)]
#[rawr(a = 1, b = "abc")]
#[rawr(c = 2, d = "def")]
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
    #[rawr(on_var = true)]
    // Comment?
    /* Comment! */
    let x = 1;
}
