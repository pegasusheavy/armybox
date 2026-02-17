# zstd-nostd

Pure Rust `#[no_std]` implementation of Zstandard (zstd) compression and decompression.

Part of the [armybox](https://github.com/pegasusheavy/armybox) project.

## Features

- Full Zstandard decompression
- Basic Zstandard compression
- `no_std` compatible — no heap allocation required (optional `alloc` feature)
- Zero external C dependencies

## Usage

```toml
[dependencies]
zstd-nostd = "0.1"
```

```rust
use zstd_nostd::decompress;

let compressed = &[/* zstd frame bytes */];
let mut output = [0u8; 4096];
let size = decompress(compressed, &mut output).unwrap();
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `alloc` | yes | Enable `Vec`-based APIs that grow output buffers automatically |

## License

MIT OR Apache-2.0
