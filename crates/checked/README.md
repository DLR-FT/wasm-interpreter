# Checked API

The core interpreter provides a relatively raw and unsafe API. This crate is a
light wrapper around the core, ensuring safety at runtime.

Essentially, by using this crate a small cost is paid at runtime to ensure
safety. The actual Wasm code interpretation is not affected by this.

The most notable reason for unsafe code at all comes from the fact that all
address types (e.g. `FuncAddr`, `GlobalAddr`, ...) are merely newtypes around
the `usize` type. As these addresses are specific to their store of origin, they
may not be used with other stores. To enforce this, we decided to use unsafe
functions in the core interpreter.
