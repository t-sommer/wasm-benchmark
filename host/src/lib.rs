//! Host helpers: locate built guest artifacts and shared test input.

use std::path::PathBuf;

pub const INPUT_LEN: usize = 100;
pub const INPUT_BYTES: usize = INPUT_LEN * 4;
pub const OUTPUT_BYTES: usize = INPUT_LEN * 8;

pub fn sample_input() -> Vec<u32> {
    (0..INPUT_LEN as u32).map(|i| i.wrapping_mul(1_103_515_245).wrapping_add(12_345)).collect()
}

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn guest_target_root() -> PathBuf {
    workspace_root().join("target-guest")
}

pub fn core_wasm_path() -> PathBuf {
    guest_target_root().join("wasm32-unknown-unknown/release/guest_core.wasm")
}

pub fn component_wasm_path() -> PathBuf {
    let root = guest_target_root();
    let component = root.join("wasm32-wasip2/release/guest_component.wasm");
    if component.exists() {
        return component;
    }
    let fallback = root.join("wasm32-unknown-unknown/release/guest_component.component.wasm");
    if fallback.exists() {
        return fallback;
    }
    root.join("wasm32-unknown-unknown/release/guest_component.wasm")
}

pub fn ensure_artifacts_exist() {
    let core = core_wasm_path();
    assert!(
        core.exists(),
        "missing core wasm at {}; run `cargo build -p host` first",
        core.display()
    );
    let component = component_wasm_path();
    assert!(
        component.exists(),
        "missing component wasm at {}; run `cargo build -p host` first",
        component.display()
    );
}

pub fn read_f64_out(memory: &[u8], out_offset: usize) -> [f64; INPUT_LEN] {
    let mut out = [0f64; INPUT_LEN];
    for (i, slot) in out.iter_mut().enumerate() {
        let start = out_offset + i * 8;
        let bytes: [u8; 8] = memory[start..start + 8].try_into().unwrap();
        *slot = f64::from_le_bytes(bytes);
    }
    out
}

pub fn write_u32_in(memory: &mut [u8], in_offset: usize, input: &[u32]) {
    for (i, &v) in input.iter().enumerate() {
        let start = in_offset + i * 4;
        memory[start..start + 4].copy_from_slice(&v.to_le_bytes());
    }
}

pub fn checksum_f64(values: &[f64]) -> f64 {
    values.iter().copied().sum()
}

/// Sanity check that core and component paths produce the same numeric result.
pub fn assert_same_result(core_sum: f64, component_sum: f64) {
    let diff = (core_sum - component_sum).abs();
    assert!(
        diff < 1e-9,
        "core vs component mismatch: core={core_sum} component={component_sum}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::{Engine, Module, Store};
    use wasmtime::component::{Component, Linker as ComponentLinker};

    wasmtime::component::bindgen!({
        path: "../wit",
        world: "convert",
    });

    struct HostState;

    #[test]
    fn core_and_component_agree() {
        ensure_artifacts_exist();
        let input = sample_input();

        let engine = Engine::default();
        let module = Module::from_file(&engine, core_wasm_path()).expect("core");
        let mut store = Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).expect("core inst");
        let memory = instance.get_memory(&mut store, "memory").expect("memory");
        let convert = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "convert")
            .expect("convert");
        let in_offset = 0usize;
        let out_offset = INPUT_BYTES;
        {
            let mut mem = memory.data_mut(&mut store);
            write_u32_in(&mut mem, in_offset, &input);
        }
        convert
            .call(&mut store, (in_offset as i32, out_offset as i32))
            .expect("convert");
        let core_sum = checksum_f64(&read_f64_out(memory.data(&store), out_offset));

        let component = Component::from_file(&engine, component_wasm_path()).expect("component");
        let linker = ComponentLinker::new(&engine);
        let mut store = Store::new(&engine, HostState);
        let bindings = Convert::instantiate(&mut store, &component, &linker).expect("inst");
        let wit_out = bindings.call_convert(&mut store, &input).expect("convert");
        let component_sum = checksum_f64(&wit_out);

        assert_same_result(core_sum, component_sum);
    }
}
