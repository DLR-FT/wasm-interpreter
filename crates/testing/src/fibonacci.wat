(module $wasm_src.wasm
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32) (local $tmp i32) (local $tmp2 i32)
    i32.const 1
    i32.const 0

    (loop (param i32) (param i32) (result i32)
      (block (param i32) (param i32) (result i32)
        local.get $n
        local.tee $tmp
        i32.eqz
        br_if 0
        local.get $tmp
        i32.const 1
        i32.sub
        local.set $n

        local.set $tmp
        local.tee $tmp2
        local.get $tmp
        i32.add
        local.get $tmp2

        br 1
      )
    )
  )
)
