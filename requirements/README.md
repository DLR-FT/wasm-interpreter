# Requirements

- The interpreter shall be "stackless"^[https://kyju.org/blog/piccolo-a-stackless-lua-interpreter/#a-stackless-interpreter-design]
- The interpreter shall be `no_std`
- The interpreter shall use no external libraries but:
  - `alloc`
  - `log`
- The interpreter shall support fuel^[https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html#method.consume_fuel] bounded execution
- The interpreter shall support epoch^[https://bytecodealliance.org/articles/wasmtime-10-performance#fast-cooperative-multitasking-with-epoch-interruption] based preemption
- The interpreter shall be resumable after being interrupted by out-of-fuel/epoch
- The interpereter shall allow instrumentation
  - Instrumentation shall enable statement coverage measurement
  - Instrumentation shall enable MC/DC measurement
    - Defmt could be a solution
- The interpreter shall allow migration of a running application to another computer
  - The interpreter state shall be serializable/deserializable in a common format
