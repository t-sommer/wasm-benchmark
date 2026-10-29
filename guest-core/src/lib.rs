//! Core WebAssembly module: `convert(in_ptr, out_ptr)` with linear memory.

use libm::sin;

const LEN: usize = 100;

#[no_mangle]
pub extern "C" fn convert(in_ptr: i32, out_ptr: i32) {
    debug_assert!(in_ptr >= 0 && out_ptr >= 0);
    unsafe {
        let input = core::slice::from_raw_parts(in_ptr as *const u32, LEN);
        let output = core::slice::from_raw_parts_mut(out_ptr as *mut f64, LEN);
        convert_slice(input, output);
    }
}

fn convert_slice(input: &[u32], output: &mut [f64]) {
    for (i, (&v, out)) in input.iter().zip(output.iter_mut()).enumerate() {
        *out = (v as f64) * 1.5 + sin(i as f64);
    }
}
