pub mod profile_7a45;

use profile_7a45::Profile7A45;

pub fn profile_for(board: &str) -> Option<Profile7A45> {
    match board {
        "7A45" => Some(Profile7A45::new()),
        _ => None,
    }
}
