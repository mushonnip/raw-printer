# raw-printer

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://github.com/mushonnip/{{raw-printer}}/workflows/ci/badge.svg?branch=release)](https://github.com/mushonnip/{{raw-printer}}/actions)
[![Docs](https://docs.rs/{{raw-printer}}/badge.svg)](https://docs.rs/{{raw-printer}})
[![crates.io](https://img.shields.io/crates/v/{{raw-printer}}.svg)](https://crates.io/crates/{{raw-printer}})

### Basic example

```rust
let _ = raw_printer::write_to_device("/dev/usb/lp0", "^FDhello world");

```

## License

[Unlicence](https://unlicense.org/)