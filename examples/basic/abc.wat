(module
    (func $add_one (param $x i32) (result i32)
        local.get $x
        i32.const 2
        i32.add)
    (export "add_one" (func $add_one))
)
