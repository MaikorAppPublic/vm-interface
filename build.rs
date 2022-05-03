#[cfg(feature = "argb")]
const ARGB: bool = true;
#[cfg(not(feature = "argb"))]
const ARGB: bool = false;
#[cfg(feature = "rgba")]
const RGBA: bool = true;
#[cfg(not(feature = "rgba"))]
const RGBA: bool = false;

fn main() {
    match (ARGB, RGBA) {
        (true, true) => panic!("Only one of the features `argb` or `rgba` can be enabled"),
        (false, false) => panic!("One of the features `argb` or `rgba` must be enabled"),
        _ => {}
    }
}
