use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use host::{
    checksum_f64, component_wasm_path, core_wasm_path, ensure_artifacts_exist, read_f64_out,
    sample_input, write_u32_in, INPUT_BYTES, INPUT_LEN,
};
use wasmtime::{Engine, Module, Store};
use wasmtime::component::{Component, Linker as ComponentLinker};

wasmtime::component::bindgen!({
    path: "../wit",
    world: "convert",
});

struct HostState;

fn bench_core(c: &mut Criterion) {
    ensure_artifacts_exist();
    let engine = Engine::default();
    let module = Module::from_file(&engine, core_wasm_path()).expect("load core module");
    let mut store = Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).expect("instantiate core");
    let memory = instance
        .get_memory(&mut store, "memory")
        .expect("exported memory");
    let convert = instance
        .get_typed_func::<(i32, i32), ()>(&mut store, "convert")
        .expect("export convert");

    let input = sample_input();
    let in_offset = 0usize;
    let out_offset = INPUT_BYTES;

    {
        let mut mem = memory.data_mut(&mut store);
        write_u32_in(&mut mem, in_offset, &input);
    }
    convert
        .call(&mut store, (in_offset as i32, out_offset as i32))
        .expect("convert");
    black_box(checksum_f64(&read_f64_out(
        memory.data(&store),
        out_offset,
    )));

    let mut group = c.benchmark_group("convert_u32_to_f64");
    group.throughput(Throughput::Elements(INPUT_LEN as u64));
    group.bench_function("core_wasm_memory", |b| {
        b.iter(|| {
            {
                let mut mem = memory.data_mut(&mut store);
                write_u32_in(&mut mem, in_offset, black_box(&input));
            }
            convert
                .call(&mut store, (in_offset as i32, out_offset as i32))
                .expect("convert");
            let sum = {
                let mem = memory.data(&store);
                checksum_f64(black_box(&read_f64_out(&mem, out_offset)))
            };
            black_box(sum)
        });
    });
    group.finish();
}

fn bench_component(c: &mut Criterion) {
    ensure_artifacts_exist();
    let engine = Engine::default();
    let component = Component::from_file(&engine, component_wasm_path()).expect("load component");
    let linker = ComponentLinker::new(&engine);
    let mut store = Store::new(&engine, HostState);
    let bindings = Convert::instantiate(&mut store, &component, &linker).expect("instantiate");

    let input = sample_input();
    let once = bindings.call_convert(&mut store, &input).expect("convert");
    black_box(checksum_f64(&once));

    let mut group = c.benchmark_group("convert_u32_to_f64");
    group.throughput(Throughput::Elements(INPUT_LEN as u64));
    group.bench_function("wit_component", |b| {
        b.iter(|| {
            let out = bindings
                .call_convert(&mut store, black_box(&input))
                .expect("convert");
            black_box(checksum_f64(black_box(&out)))
        });
    });
    group.finish();
}

criterion_group!(benches, bench_core, bench_component);
criterion_main!(benches);
