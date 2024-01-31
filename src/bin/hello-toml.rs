// SPDX-License-Identifier: Apache-2.0

//! Parse the basic config toml. The `toml` crate seems to pull in serde.
//! Follow along with https://www.makeuseof.com/working-with-toml-files-in-rust/

use anyhow::bail;
use regex::Regex;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::cell::OnceCell;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use toml::{self, Table};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    if args.len() != 2 {
        bail!("Usage: hello-toml FILE.toml");
    }

    let path = args.nth(1).unwrap();
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse without Serde
    let _config = contents.parse::<Table>()?;

    // language=toml
    let inline = r#"
    [rawr]
    description = "This is a plain string"
    # Looks like this needs a custom deserializer
    foo = 'D("foo", 12)'
    "#;

    let config: Config = toml::from_str(inline)?;
    dbg!(config);

    Ok(())
}

/* Syntax:
 * A
 * ^B(arg(,\s* arg)*)$
 *   I think toml::de::ValueDeserializer has the required functionality
 */
#[derive(Debug)]
pub enum Foo {
    A,
    B { count: u64 },
    C(String),
    D(u64, String),
}

// Starting to build a lightly-customized deserializer.
// see: https://github.com/serde-rs/serde/issues/2086
impl<'de> Deserialize<'de> for Foo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO Convert to once/static
        let regex = Regex::new(r"^(?P<variant>A|B|C|D)(?P<bracketed_args>\((?P<args>.+?)\))?$")
            .expect("Compile regular expression for Foo variants");

        let s = String::deserialize(deserializer)?;
        let Some(matches) = regex.captures(&s) else {
            return Err(de::Error::custom(
                "Invalid format. Expected a variant of Foo.",
            ));
        };

        let Some(variant) = matches.name("variant") else {
            return Err(de::Error::custom(
                "Unknown variant, or variant not specified",
            ));
        };

        let bracketed_args = matches.name("bracketed_args");
        let args = matches.name("args");

        match variant.as_str() {
            "A" => {
                return Ok(Foo::A);
            }
            "B" => {
                if !bracketed_args.is_none() || args.is_none() {
                    return Err(de::Error::missing_field("count"));
                }

                let args = args.unwrap().as_str();
                let Ok(count) = u64::from_str(args) else {
                    return Err(de::Error::custom("Count must be a u64"));
                };

                return Ok(Foo::B { count });
            }
            "C" => todo!("Parse variant C"),
            "D" => todo!("Parse variant D"),
            unknown @ _ => {
                return Err(de::Error::unknown_variant(
                    unknown,
                    &["A", "B(u64)", "C(String)", "D(u64, String)"],
                ))
            }
        };
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rawr: RawrSection,
}

#[derive(Deserialize, Debug)]
pub struct RawrSection {
    pub description: String,
    // This is actually a String containing a Foo. How do we deserialize this?
    pub foo: Option<Foo>,
}
