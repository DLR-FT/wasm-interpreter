(module
    (import "env" "add" (func $add (param i32 i32) (result i32)))
    (import "env" "mul" (func $mul (param i32 i32) (result i32)))

    ;; Returns a * b + c
    (func (export "mul_then_add") (param $a i32) (param $b i32) (param $c i32) (result i32)
        local.get $a
        local.get $b
        call $mul
        local.get $c
        call $add
    )
)
