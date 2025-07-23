use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};

use wasm::{validate, RuntimeInstance};

macro_rules! bench_wasm {
    {
        name = $name:ident;
        plot_config = $plot_config:expr;
        wat = $wat:expr;
        entry_function = $entry_function:expr;
        arg_type = $arg_type:ty;
        return_type = $return_type:ty;
        inputs = $inputs:expr;
    } => {
        bench_wasm!{
            name = $name;
            plot_config = $plot_config;
            wasm_bytes = {
                let wat = $wat;
                let wasm_bytes = wat::parse_str(wat).unwrap();
                wasm_bytes
            };
            entry_function = $entry_function;
            arg_type = $arg_type;
            return_type = $return_type;
            inputs = $inputs;
        }
    };

    {
        name = $name:ident;
        plot_config = $plot_config:expr;
        wasm_bytes = $wasm_bytes:expr;
        entry_function = $entry_function:expr;
        arg_type = $arg_type:ty;
        return_type = $return_type:ty;
        inputs = $inputs:expr;
    } => {

        fn $name(c: &mut Criterion) {
            let plot_config = $plot_config;
            let wasm_bytes = $wasm_bytes;

            // Our interpreter
            let our_validation_info = validate(&wasm_bytes).unwrap();
            let mut our_instance = RuntimeInstance::new(&our_validation_info).unwrap();
            let our_fn = our_instance
                .get_function_by_name(wasm::DEFAULT_MODULE, $entry_function)
                .unwrap();

            // Wasmtime
            let wasmtime_engine = wasmtime::Engine::default();
            let wasmtime_module = wasmtime::Module::new(&wasmtime_engine, &wasm_bytes).unwrap();
            let wasmtime_linker = wasmtime::Linker::new(&wasmtime_engine);
            let mut wasmtime_store: wasmtime::Store<()> = wasmtime::Store::new(&wasmtime_engine, ());
            let wasmtime_instance = wasmtime_linker
                .instantiate(&mut wasmtime_store, &wasmtime_module)
                .unwrap();
            let wasmtime_fn = wasmtime_instance
                .get_typed_func::<$arg_type, $return_type>(&mut wasmtime_store, $entry_function)
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
                .get_typed_func::<$arg_type, $return_type>(&wasmi_store, $entry_function)
                .unwrap();

            //
            // Actual Benchmark
            //

            let mut group = c.benchmark_group(stringify!($name));
            group.plot_config(plot_config);

            for n in $inputs {
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
                        our_instance.invoke::<$arg_type, $return_type>(&our_fn, s).unwrap();
                    })
                });
            }
            group.finish();
        }
    };
}

bench_wasm! {
    name = fibonacci_recursive;
    plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    // Source: <https://stackoverflow.com/a/53416725>
    wat = r#"
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
    entry_function = "fib";
    arg_type = i32;
    return_type = i32;
    inputs = (0..=9).map(|p| 1 << p);
}

}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));
    targets = fibonacci_recursive
}

criterion_main!(benches);
