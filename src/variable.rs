//! The two components used are Variables and Wires.
use std::fmt::Display;

/// The value is a reference to the actual value that was added to the
/// constraint system
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Variable(pub(crate) usize);

impl Display for Variable {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stores the data for a specific wire in an arithmetic circuit
// This data is the gate index and the type of wire
// Left(1) signifies that this wire belongs to the first gate and is the left wire
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireData {
    Left(usize),
    Right(usize),
    Output(usize),

}
