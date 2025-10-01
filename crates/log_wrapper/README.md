# Overview

The purpose of this crate is to provide a configurable logging backend, whose logging macros either resolve to nothing (empty statements), or to a downstream [`log`](https://docs.rs/log/) implementation.
