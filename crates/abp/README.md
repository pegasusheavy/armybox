# abp

ABP (ArmyBox Package) manager — a minimal, secure package manager for armybox-based Linux systems.

Part of the [armybox](https://github.com/pegasusheavy/armybox) project.

## Features

- `#[no_std]` compatible core (with optional `alloc` feature)
- Binary package format with Ed25519 signatures and zstd-compressed payloads
- Compact binary package index (`.abpd`) with O(log n) lookups and zero-copy reads
- Dependency resolution and transaction support
- ABUILD-based package building (similar to Alpine's `abuild`)

## Crate Modules

| Module | Description |
|--------|-------------|
| `pkgindex` | Compact binary package index — `no_std` reader + `alloc` builder |
| `format` | `.abp` package format parser |
| `database` | Local installed-package database |
| `install` | Package installation logic |
| `remove` | Package removal logic |
| `resolver` | Dependency resolution |
| `crypto` | Ed25519 signatures + SHA-256 |
| `makepkg` | Build packages from ABUILD files |

## Binary: `abpd-gen`

Generates `.abpd` binary package indexes from tab-separated input:

```sh
# Pipe package data and write binary index
cat packages.tsv | abpd-gen output/packages.abpd
```

## License

MIT OR Apache-2.0
