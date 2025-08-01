# wasm-interpreter

<p align="center">
  <a href="https://dlr-ft.github.io/wasm-interpreter/main/">Website</a> &nbsp;&bull;&nbsp;
  <a href="#features">Features</a> &nbsp;&bull;&nbsp;
  <a href="#resources">Resources</a>
</p>
<p align="center">
  <a href="https://github.com/DLR-FT/wasm-interpreter/actions/workflows/nix.yaml"><img src="https://github.com/DLR-FT/wasm-interpreter/actions/workflows/nix.yaml/badge.svg" alt="ci status" /></a>
  <a href="https://app.codecov.io/github/dlr-ft/wasm-interpreter"><img src="https://img.shields.io/codecov/c/github/DLR-FT/wasm-interpreter" alt="code coverage" /></a>
  <a href="https://dlr-ft.github.io/wasm-interpreter/main/rustdoc/wasm"><img src="https://img.shields.io/badge/rustdoc-passing-orange" alt="license" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue" alt="license" /></a>
</p>

A minimal in-place interpreter for [WebAssembly](https://webassembly.org/) bytecode (almost without) dependencies while being `no_std`.

## Features

- **In-place interpretation**: No intermediate parsing step. This allows for fast start-up times.
- **`no_std` support**: The interpreter requires only Rust's `core` and `alloc` libraries allowing its use in various environments, such as bare-metal systems.
- **Minimal dependencies**: Only two dependencies (`log` and `libm`) are currently required by the interpreter.
- **Specification compliance**: The interpreter passes (almost) all test from the [official WebAssembly testsuite](https://github.com/WebAssembly/testsuite)

_For information on other features, visit our [requirements page](https://dlr-ft.github.io/wasm-interpreter/main/requirements/html/index.html)._

### Planned

- **Fuel & resumable execution**: A fuel mechanism is used to halt execution once fuel runs out. Then fuel can be refilled and execution resumed.
- **C Bindings**: The interpreter can be used from C code.
- **Host functions**: A host system can provide functions for Wasm code to call.
- **Migratability**: Wasm instances can be transferred between systems during their execution.

### Not planned

Multi-memory proposal, GC proposal

## Resources

- `A fast in-place interpreter` by Ben L. Titzer: https://arxiv.org/abs/2205.01183
- WebAssembly spec: https://webassembly.github.io/spec/core/index.html
- WebAssembly Opcode Table: https://pengowray.github.io/wasm-ops/
- Compiler/Interpreter Know-How Gist Compilation: https://gist.github.com/o11c/6b08643335388bbab0228db763f99219
- Mozilla Developer Network WebAssembly Homepage: https://developer.mozilla.org/en-US/docs/WebAssembly

## License

Licensed under either of

- APACHE Version 2.0, [LICENSE-APACHE](LICENSE-APACHE)
- MIT LICENSE, [LICENSE-MIT](LICENSE-MIT)

at your option.
