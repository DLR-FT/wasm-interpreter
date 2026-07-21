# Examples

- **[`function_invocation`](function_invocation.rs)**: [RECOMMENDED FIRST] Instantiation and
  function invocation with a simple Wasm module.
- **[`embedder_api`](embedder_api.rs)**: The Embedder API
- **[`fuel`](fuel.rs)**: Using fuel to preempt a fibonacci computation regulary combined with progress reporting via an exported memory.
- **[`fuel_configuration`](fuel_configuration.rs)**: Configuration of fuel consumption per instruction
- **[`host_functions`](host_functions.rs)**: Allocate host functions into the store and invoke them
- **[`host_functions_with_registry`](host_functions_with_registry.rs)**: Using the registry utility crate for host functions
- **[`instruction_hook`](instruction_hook.rs)**: A hook executed on every instruction
- **[`linking`](linking.rs)**: Linking multiple Wasm modules together
