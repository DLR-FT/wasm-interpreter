use criterion::{black_box, criterion_group, criterion_main, Criterion};

use wasm::{hooks::HookSet, validate, RuntimeInstance, DEFAULT_MODULE};

fn criterion_benchmark(c: &mut Criterion) {
    let wat = r#"
    (module
        (memory 1)
        (func $add_one (param $x i32) (result i32) (local $ununsed_local i32)
            local.get $x
            i32.const 1
            i32.add)

        (func $add (param $x i32) (param $y i32) (result i32)
            local.get $y
            local.get $x
            i32.add)

        (func (export "store_num") (param $x i32)
            i32.const 0
            local.get $x
            i32.store)
        (func (export "load_num") (result i32)
            i32.const 0
            i32.load)

        (export "add_one" (func $add_one))
        (export "add" (func $add))
    )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let validation_info = validate(&wasm_bytes).unwrap();

    //
    // Set up an interpreter with the empty hook-set
    //
    let mut instance_empty_hookset = RuntimeInstance::new(&validation_info).unwrap();

    //
    // Set up an interpreter with an non-empty hook-set
    //
    #[derive(Default, Debug)]
    struct MyCustomHookSet;
    impl HookSet for MyCustomHookSet {
        fn instruction_hook(&mut self, bytecode: &[u8], pc: usize) {
            if black_box(bytecode[pc]) == 0x20 {
                eprintln!("First instruction is a local.get");
            }
        }
    }

    let mut instance_non_empty_hookset =
        RuntimeInstance::new_with_hooks(DEFAULT_MODULE, &validation_info, MyCustomHookSet).unwrap();

    let test_fn = instance_empty_hookset.get_function_by_index(0, 2).unwrap();
    c.bench_function("invoke_func EmptyHookSet", |b| {
        b.iter(|| instance_empty_hookset.invoke_typed::<_, ()>(&test_fn, black_box(42_i32)))
    });

    let test_fn = instance_non_empty_hookset
        .get_function_by_index(0, 2)
        .unwrap();
    c.bench_function("invoke_func MyCustomHookSet", |b| {
        b.iter(|| instance_non_empty_hookset.invoke_typed::<_, ()>(&test_fn, black_box(42_i32)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
