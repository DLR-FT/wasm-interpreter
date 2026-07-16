# wasm-interpreter

<p align="center">
  <a href="https://dlr-ft.github.io/wasm-interpreter/main/">Website</a> &nbsp;&bull;&nbsp;
  <a href="#features">Features</a> &nbsp;&bull;&nbsp;
  <a href="#our-works">Our Works</a> &nbsp;&bull;&nbsp;
  <a href="#resources">Resources</a>
</p>
<p align="center">
  <a href="https://github.com/DLR-FT/wasm-interpreter/actions/workflows/nix.yaml"><img src="https://github.com/DLR-FT/wasm-interpreter/actions/workflows/nix.yaml/badge.svg" alt="ci status" /></a>
  <a href="https://app.codecov.io/github/DLR-FT/wasm-interpreter"><img src="https://codecov.io/gh/DLR-FT/wasm-interpreter/graph/badge.svg?component=interpreter" alt="code coverage" /></a>
  <a href="https://dlr-ft.github.io/wasm-interpreter/main/rustdoc/wasm"><img src="https://img.shields.io/badge/rustdoc-passing-orange" alt="license" /></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20or%20Apache%202.0-blue" alt="license" /></a>
</p>

A minimal in-place interpreter for [WebAssembly 2.0](https://webassembly.org/) bytecode (almost without) dependencies while being `no_std`.

## Features

- **In-place interpretation**: No intermediate representation, directly interprets WebAssembly bytecode. This allows for fast start-up times.
- **`no_std` support**: The interpreter requires only Rust's `core` and `alloc` libraries allowing its use in various environments, such as bare-metal systems.
- **Minimal dependencies**: The interpreter requires only one dependency: `libm`. `log` is also supported but disabled by default.
- **Compliance with specification**: The interpreter passes all tests from the [official WebAssembly testsuite](https://github.com/WebAssembly/testsuite), except for the unfinished proposal tests. See [`GlobalConfig` in `tests/specification/mod.rs`](tests/specification/mod.rs) for the default spec-test filter regex.
- **Returning host functions**: The host system can provide functions for Wasm code to call. Contrary to other Wasm runtimes, host functions are not owned by the interpreter. Instead control flow is returned back to the user, when Wasm code calls a host function.
- **Fuel & resumable execution**: A fuel mechanism is used to halt execution once fuel runs out. Then fuel can be refilled and execution resumed.

_For information on other features, visit our [requirements page](https://dlr-ft.github.io/wasm-interpreter/main/requirements/html/index.html)._

### Planned

- **C bindings**: The interpreter can be used from C code.
- **Migratability**: Wasm instances can be transferred between systems during their execution.
- **Threading**: Support for the [Threads Proposal](https://github.com/WebAssembly/threads) is work-in-progress. The [Shared-Everything Threads Proposal](https://github.com/WebAssembly/shared-everything-threads) is also of interest.
- **Flexible Allocation API**: Custom Allocators should be configurable. There is still some uncertainty, especially regarding the use of the Rust Allocator API.

### Not planned

- GC proposal

## Our Works

Click to show more information:

<details>
<summary>WebAssembly in Avionics: Decoupling Software from Hardware</summary>

Link (Open Access): <https://elib.dlr.de/201323/>

```bibtex
@INPROCEEDINGS{zaeske_wasm_2023,
  author={Zaeske, Wanja and Friedrich, Sven and Schubert, Tim and Durak, Umut},
  booktitle={2023 IEEE/AIAA 42nd Digital Avionics Systems Conference (DASC)},
  title={WebAssembly in Avionics: Decoupling Software from Hardware},
  year={2023},
  volume={},
  number={},
  pages={1-10},
  url={https://elib.dlr.de/201323/},
  keywords={Couplings;Virtual machine monitors;Full stack;Aerospace electronics;Webassembly;Software;Hardware;Virtual machines;Certification;Application programming interfaces;Avionics;Wasm;ARINC 653;Software},
  doi={10.1109/DASC58513.2023.10311207}
}
```

</details>

<details>
<summary>On the Design of a WebAssembly Interpreter for Safety Critical Avionics Applications</summary>

Link (Open Access): <https://elib.dlr.de/219593>

```bibtex
@INPROCEEDINGS{zaeske_wasm_2025,
  author={Zaeske, Wanja and Önem, A. Cem and Hartung, Florian and Durak, Umut},
  booktitle={2025 AIAA DATC/IEEE 44th Digital Avionics Systems Conference (DASC)},
  title={On the Design of a WebAssembly Interpreter for Safety Critical Avionics Applications},
  year={2025},
  volume={},
  number={},
  pages={1-10},
  url={https://elib.dlr.de/219593/},
  keywords={Codes;Instruction sets;Aerospace electronics;Webassembly;Software;Hardware;Safety;Space exploration;Certification;Standards;Avionics;Wasm;ED-12C/DO-178C;ED-217/DO-332},
  doi={10.1109/DASC66011.2025.11257180}
}
```

</details>

## Resources

- [A fast in-place interpreter](https://dl.acm.org/doi/10.1145/3563311) by Ben L. Titzer
- WebAssembly: [Website](https://webassembly.org/), [Spec 2.0 on W3C](https://www.w3.org/TR/2025/CRD-wasm-core-2-20250616/), [Spec 2.0 as PDF](https://webassembly.github.io/spec/versions/core/WebAssembly-2.0.pdf),
- [Mozilla Developer Network - WebAssembly Homepage](https://developer.mozilla.org/en-US/docs/WebAssembly)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Copyright

Copyright © 2024-2026 Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR)
Copyright © 2024-2025 OxidOS Automotive SRL
