# raw-printer

[![Docs](https://docs.rs/raw-printer/badge.svg)](https://docs.rs/raw-printer)
[![crates.io](https://img.shields.io/crates/v/raw-printer.svg)](https://crates.io/crates/raw-printer)

### Basic example

```rust
let _ = raw_printer::write_to_device("/dev/usb/lp0", "^FDhello world");

```

## License

[MIT](https://github.com/aws/mit-0)
