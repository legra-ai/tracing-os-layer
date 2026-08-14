# tracing-os-layer

[![Crates.io](https://img.shields.io/crates/v/tracing-os-layer.svg)](https://crates.io/crates/tracing-os-layer)
[![Documentation](https://docs.rs/tracing-os-layer/badge.svg)](https://docs.rs/tracing-os-layer)
[![CI](https://github.com/legra-ai/tracing-os-layer/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/tracing-os-layer/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tracing-os-layer.svg)](https://github.com/legra-ai/tracing-os-layer/blob/main/LICENSE-APACHE)

A cross-platform [`tracing`](https://docs.rs/tracing) layer for native
operating-system logging.

## Backends

`OsLogLayer` selects a backend at compile time:

- macOS uses Apple Unified Logging (`os_log`) and Activity Tracing
  (`os_activity`). Activity scopes are entered and left inside `on_event`, so
  they never span asynchronous task migration between threads.
- Linux uses [`tracing-journald`](https://docs.rs/tracing-journald) and returns
  `None` when the systemd journal socket is unavailable.
- Windows uses the Windows Event Log Application log through
  the `eventlog` crate's native backend and embedded message resources. The
  `subsystem` must already be registered as an event source, normally by an
  installer, with the application executable as its message file;
  `try_new` returns `None` when Windows cannot open it. The `category` is
  included in each event message.
- Other platforms expose the same API as a no-op layer and return `None`.

## Example

```rust
use tracing_subscriber::prelude::*;
use tracing_os_layer::OsLogLayer;

let native = OsLogLayer::try_new("com.example.app", "worker");
tracing_subscriber::registry()
    .with(native)
    .init();

tracing::info!(records = 12, "processed input");
```

The returned `Option` can be passed directly to a subscriber because
`Option<L>` implements `Layer<S>`: unavailable native logging is simply left
out of the subscriber stack.

## Scope

This crate only adapts `tracing` events and spans to native OS logging. It does
not configure global subscribers, choose application identifiers, or impose a
logging policy.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT)
  or <https://opensource.org/licenses/MIT>)

at your option.
