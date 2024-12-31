use rawr_attribute::rawr;

// Matching against anything that's not in a bash function or variable is a
// little bit problematic.
fn main() -> anyhow::Result<()> {
    println!("Testing!");
    Ok(())
}

#[rawr(
    file = "tests/upstream.sh",
    kind = "variable",
    name = "FOO",
    rev = "2787b552e08c23c64463ee98bf05469a8c2df4c5"
)]
const FOO: usize = 1;

#[rawr(
    file = "tests/upstream.sh",
    kind = "variable",
    name = "BAR",
    rev = "7d2cada3b43777287a35062f6f43bd1cbac66250"
)]
const BAR: usize = 2;

/// Accurately count number of light sources
#[rawr(
    file = "tests/upstream.sh",
    kind = "function",
    name = "foo",
    rev = "5a2933f5c81bc86a594d535dc8cc413aa80ef0fe"
)]
fn foo(bar: Option<usize>) {
    let bar = bar.map(usize::to_string).unwrap_or("no".to_string());
    println!("There are {bar} lights")
}
