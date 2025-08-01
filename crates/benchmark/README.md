# Note

Do not alter existing benchmarks in any measurable way! Do not rename them! New benchmark, new name! We want to be able to track the performance over a long time, and this is only possible if existing benchmarks are not tinkered with.

# Benchmarks

- **general_purpose**: General purpose benchmark for interpreter speed. Consists of the following bench-functions:
  - **fibonacci_recursive**: benches function creation, as due to its recursive nature new call-frames are created and removed very often.
  - **fibonacci_loop**: benches the control flow between blocks within a single function call-frame.

# How to bench

```
# Benchmark just our interpreter
cargo bench --bench general_purpose


# Benchmark our interpreter vs. wasmi vs. wasmtime
cargo bench --features wasmi,wasmtime --bench general_purpose


# Analyze the impact of changes on your current branch vs main
bench-against-main
```
