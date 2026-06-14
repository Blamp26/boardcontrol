pub mod allowlist;
pub mod rmw;
pub mod sequence;
pub mod superio;
pub mod types;

use crate::backend::Backend;
use crate::error::Result;
use sequence::NctSequence;

pub fn run_sequence<B: Backend>(backend: &mut B, sequence: &NctSequence) -> Result<()> {
    sequence.execute(backend)
}
