use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};

use wasm::{validate, RuntimeInstance};

// Bench the interpreter with a recusive version of an algorithm to find fibonacci numbers
fn bench_fibonacci_recursive(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    // Wat code borrowed from <https://stackoverflow.com/a/53416725>
    let wat = r#"
(module
  (func $fib2 (param $n i32) (param $a i32) (param $b i32) (result i32)
    (if (result i32)
        (i32.eqz (local.get $n))
        (then (local.get $a))
        (else (call $fib2 (i32.sub (local.get $n)
                                   (i32.const 1))
                          (local.get $b)
                          (i32.add (local.get $a)
                                   (local.get $b))))))

  (func $fib (param i32) (result i32)
    (call $fib2 (local.get 0)
                (i32.const 0)   ;; seed value $a
                (i32.const 1))) ;; seed value $b

  (export "fib" (func $fib)))
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    // Our interpreter
    let our_validation_info = validate(&wasm_bytes).unwrap();
    let mut our_instance = RuntimeInstance::new(&our_validation_info).unwrap();
    let our_fn = our_instance.get_function_by_index(0, 1).unwrap();

    // Wasmtime
    let wasmtime_engine = wasmtime::Engine::default();
    let wasmtime_module = wasmtime::Module::new(&wasmtime_engine, &wasm_bytes).unwrap();
    let wasmtime_linker = wasmtime::Linker::new(&wasmtime_engine);
    let mut wasmtime_store: wasmtime::Store<()> = wasmtime::Store::new(&wasmtime_engine, ());
    let wasmtime_instance = wasmtime_linker
        .instantiate(&mut wasmtime_store, &wasmtime_module)
        .unwrap();
    let wasmtime_fn = wasmtime_instance
        .get_typed_func::<i32, i32>(&mut wasmtime_store, "fib")
        .unwrap();

    // Wasmi
    let wasmi_engine = wasmi::Engine::default();
    let wasmi_module = wasmi::Module::new(&wasmi_engine, &wasm_bytes).unwrap();
    let mut wasmi_store = wasmi::Store::new(&wasmi_engine, ());
    let wasmi_linker = <wasmi::Linker<()>>::new(&wasmi_engine);
    let wasmi_instance = wasmi_linker
        .instantiate(&mut wasmi_store, &wasmi_module)
        .unwrap()
        .start(&mut wasmi_store)
        .unwrap();
    let wasmi_fn = wasmi_instance
        .get_typed_func::<i32, i32>(&wasmi_store, "fib")
        .unwrap();

    //
    // Actual Benchmark
    //
    let ns: Vec<i32> = (0..=9).map(|p| 1 << p).collect();

    let mut group = c.benchmark_group("fibonacci_recursive");
    group.plot_config(plot_config);

    for n in ns.into_iter() {
        group.throughput(Throughput::Elements(n as u64));

        let bid = BenchmarkId::new("wasmtime", n);
        group.bench_with_input(bid, &n, |b, &s| {
            b.iter(|| wasmtime_fn.call(&mut wasmtime_store, s).unwrap());
        });

        let bid = BenchmarkId::new("wasmi", n);
        group.bench_with_input(bid, &n, |b, &s| {
            b.iter(|| wasmi_fn.call(&mut wasmi_store, s).unwrap());
        });

        let bid = BenchmarkId::new("our", n);
        group.bench_with_input(bid, &n, |b, &s| {
            b.iter(|| {
                our_instance.invoke::<i32, i32>(&our_fn, s).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));
    targets = bench_fibonacci_recursive
}

criterion_main!(benches);
