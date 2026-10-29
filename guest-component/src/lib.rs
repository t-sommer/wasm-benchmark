//! Component-model guest: same logic as `guest-core`, exported through WIT.

use libm::sin;

wit_bindgen::generate!({
    world: "convert",
    path: "../wit",
});

struct GuestImpl;

impl Guest for GuestImpl {
    fn convert(values: Vec<u32>) -> Vec<f64> {
        values
            .iter()
            .enumerate()
            .map(|(i, &v)| (v as f64) * 1.5 + sin(i as f64))
            .collect()
    }
}

export!(GuestImpl);
