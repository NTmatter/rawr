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
struct S {
    a: Option<String>,
}
