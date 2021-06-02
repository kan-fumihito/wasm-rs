use serde::{Serialize, Deserialize};
#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Byte(pub u8);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Name(pub String);
