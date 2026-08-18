# Android/Bionic Support for the Full armybox Binary

**Date:** 2026-08-18
**Status:** Approved
**Goal:** Full applet parity on Android with `aarch64-linux-android` as a release target.

## Background

Commit `ad53b10e` (PR #5) fixed `c_char`, `mode_t`, `time_t`, and errno portability,
which made `crates/abp` build cleanly for Android and fixed the musl ARM release
targets. The full armybox binary still fails on `aarch64-linux-android` with 59
errors in two categories:

1. **17 missing-symbol errors** — the libc crate does not export these for
   Android: `reboot`/`RB_*` (halt, poweroff, reboot applets), `struct rtentry`
   and `RTF_*` (route), `SCHED_OTHER` (chrt), `gethostid` (hostid),
   `_PC_FILESIZEBITS` (getconf). Some exist in Bionic but are unbound in the
   libc crate; `gethostid` is genuinely absent from Bionic.
2. **~42 mismatched-type errors** — one root cause: Bionic's `ioctl()` takes
   `c_int` as the request argument where glibc/musl take `c_ulong`. Affects
   blockdev, blkdiscard, chvt, deallocvt, eject, freeramdisk, fsfreeze, gpio*,
   i2c*, openvt, partprobe, screen, init, route, and `personality()` in linux32.

## Decisions

- **Approach:** local compat layer (Approach A). No waiting on upstream libc
  releases; upstreaming bindings later is an optional cleanup, not a dependency.
- **Targets:** `aarch64-linux-android` only in the release matrix. Other ABIs
  build locally with cargo-ndk if needed.
- **Applets:** no applet is gated out on Android. Full parity.

## Design

### 1. Compat layer in `src/sys.rs`

All additions follow the cfg-split pattern `sys::errno()` already uses.

- `pub type IoctlReq` — `libc::c_int` on `target_os = "android"`,
  `libc::c_ulong` otherwise. Fixes the ~42 ioctl errors via call-site casts.
- `pub fn reboot(cmd: i32) -> i32` — calls `libc::reboot(cmd)` off-Android; on
  Android issues `libc::syscall(SYS_reboot, LINUX_REBOOT_MAGIC1,
  LINUX_REBOOT_MAGIC2, cmd, 0)`. `RB_AUTOBOOT` (0x01234567),
  `RB_HALT_SYSTEM` (0xcdef0123), and `RB_POWER_OFF` (0x4321fedc) are defined
  under cfg for Android — kernel ABI values from `linux/reboot.h`, identical on
  every Linux.
- `pub fn gethostid() -> i64` — calls `libc::gethostid()` off-Android; on
  Android (no gethostid in Bionic) uses busybox semantics: read `/etc/hostid`
  if present, else return 0.
- constants `SCHED_OTHER`, `_PC_FILESIZEBITS`, `RB_AUTOBOOT`,
  `RB_HALT_SYSTEM`, `RB_POWER_OFF`: `sys.rs` re-exports the libc crate's
  values off-Android (`pub use libc::...`) and defines them under
  `#[cfg(target_os = "android")]` otherwise, so applets reference `sys::` on
  every platform. Bionic has `SCHED_OTHER` and `_PC_FILESIZEBITS`; the libc
  crate just does not bind them for Android. Values are taken from the NDK
  headers (`sched.h`, `bits/posix_limits.h`/`unistd.h`) and verified during
  implementation, not assumed.

Applets `halt`, `poweroff`, `reboot`, `hostid`, `chrt`, and `getconf` switch
from the missing `libc::` items to the `sys::` equivalents on all platforms
(one code path; the cfg lives in `sys.rs`).

### 2. `rtentry` in `src/applets/network/route.rs`

`struct rtentry`, `RTF_UP` (0x0001), `RTF_GATEWAY` (0x0002), and `RTF_HOST`
(0x0004) are kernel ABI from `linux/route.h`, identical on every Linux. Under
`#[cfg(target_os = "android")]`, define a `#[repr(C)]` `rtentry` and the three
constants locally in `route.rs`; under `#[cfg(not(target_os = "android"))]`,
`use libc::rtentry`. They stay in `route.rs` because it is the only consumer.
The struct layout must match libc's `rtentry` field-for-field so both cfg arms
share the same construction code.

### 3. Applet ioctl sweep

Mechanical pass over the affected files: `libc::ioctl(fd, REQ, ...)` becomes
`libc::ioctl(fd, REQ as sys::IoctlReq, ...)`. On non-Android targets the cast
is `c_ulong as c_ulong` — a no-op — so host behavior is unchanged by
construction. Individually handled:

- `personality(PER_LINUX32)` in `linux32.rs`: cast to the platform's parameter
  type (Bionic's signature differs from glibc/musl here too).
- Any ioctl request constants the libc crate turns out not to bind on Android
  get local cfg-gated definitions next to their use. These are discovered by
  iterating `cargo check --target aarch64-linux-android` until clean, since the
  current 59 errors stop at the first failing layer.

### 4. Release matrix

Add to `.github/workflows/release.yml`:

```yaml
- target: aarch64-linux-android
  os: ubuntu-latest
  cross: true
```

cross-rs ships an NDK-based image for this target, so it slots in exactly like
the musl ARM entries — both the `armybox` binary and `abpd-gen` build steps
apply unchanged. No cargo-ndk plumbing in CI.

## Error handling

- `sys::reboot` and `sys::gethostid` preserve the current call-site contracts:
  return `-1`/errno on failure like the libc functions they replace, so applet
  error paths need no changes.
- The `/etc/hostid` fallback treats an unreadable or short file as absent
  (return 0) rather than erroring — matching busybox.

## Testing and verification

- Gate: `cargo check --release --target aarch64-linux-android
  --no-default-features --features "abp,alloc,full"` at 0 errors.
- Regression: host `cargo build --release` and the full release-profile test
  suite (currently 1468 passing) stay green. The sweep is behaviorally inert
  off-Android, so existing tests cover it.
- Existing targets: `aarch64-unknown-linux-musl` and
  `armv7-unknown-linux-musleabihf` release checks stay at 0 errors.
- Runtime smoke test (`adb push` + run on a device or emulator) stays manual;
  CI has no emulator.

## Out of scope

- Other Android ABIs in CI (arm32, x86_64, x86).
- Upstreaming the missing bindings to rust-lang/libc (optional follow-up).
- SELinux/permission behavior of applets at runtime on Android; this design is
  about building, and runtime capability is bounded by the device's policy.
