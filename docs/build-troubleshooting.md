# Troubleshooting Guide: Rust Native & WASM Builds

This document covers common issues encountered during Rust native builds, WASM target builds,
and React Native integration. Follow the steps below to diagnose and resolve them.

---

## Table of Contents

1. [Missing Rust Target](#1-missing-rust-target)
2. [Cargo Build Failures](#2-cargo-build-failures)
3. [WASM Target Build Issues](#3-wasm-target-build-issues)
4. [React Native Integration Notes](#4-react-native-integration-notes)
5. [General Tips](#5-general-tips)

---

## 1. Missing Rust Target

### Symptom
```
error[E0463]: can't find crate for `std`
error: cannot find crate for `std`
note: the `wasm32-unknown-unknown` target may not be installed
```
or
```
error: toolchain 'stable-x86_64-unknown-linux-gnu' does not contain component 'rust-std' for target 'aarch64-linux-android'
```

### Cause
The required compilation target has not been added to your Rust toolchain.

### Fix

**Check currently installed targets:**
```bash
rustup target list --installed
```

**Add the missing target:**

For WASM:
```bash
rustup target add wasm32-unknown-unknown
```

For Android (React Native):
```bash
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

For iOS (React Native):
```bash
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
rustup target add aarch64-apple-ios-sim
```

**Verify the target was added:**
```bash
rustup target list --installed
```

---

## 2. Cargo Build Failures

### 2a. Linker Not Found

**Symptom:**
```
error: linker `cc` not found
  = note: No such file or directory (os error 2)
```

**Fix:**

On Ubuntu/Debian:
```bash
sudo apt-get install build-essential
```

On macOS:
```bash
xcode-select --install
```

On Windows: Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "C++ build tools" workload.

---

### 2b. Dependency Version Conflicts

**Symptom:**
```
error: failed to select a version for `some-crate`
```

**Fix:**
```bash
# Update all dependencies
cargo update

# If a specific crate is the issue, check Cargo.lock
cargo tree | grep <crate-name>
```

If the conflict persists, check `Cargo.toml` for incompatible version constraints and align them manually.

---

### 2c. Out-of-date Toolchain

**Symptom:**
```
error: package `xyz v1.x.x` cannot be built because it requires rustc 1.xx.0 or newer
```

**Fix:**
```bash
# Update Rust to the latest stable version
rustup update stable

# Verify version
rustc --version
```

---

### 2d. Missing System Libraries

**Symptom:**
```
error: failed to run custom build command for `openssl-sys`
```

**Fix:**

On Ubuntu/Debian:
```bash
sudo apt-get install libssl-dev pkg-config
```

On macOS:
```bash
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

---

### 2e. Corrupt Cargo Cache

**Symptom:** Unexplained build failures, especially after interrupted downloads.

**Fix:**
```bash
# Clean the project build artifacts
cargo clean

# Clear the global Cargo registry cache (use with caution)
rm -rf ~/.cargo/registry/cache
```

Then rebuild:
```bash
cargo build
```

---

## 3. WASM Target Build Issues

### 3a. wasm-pack Not Installed

**Symptom:**
```
command not found: wasm-pack
```

**Fix:**
```bash
cargo install wasm-pack
```

Or via the installer script:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

---

### 3b. wasm-bindgen Version Mismatch

**Symptom:**
```
it looks like the Rust project used to create this wasm file was linked against
version of wasm-bindgen that uses a different binding generation format
```

**Fix:**

Ensure `wasm-bindgen` versions in `Cargo.toml` and `wasm-pack` CLI are aligned:
```bash
# Check installed version
wasm-pack --version
cargo install wasm-bindgen-cli --version <matching-version>
```

Or update `Cargo.toml` to match your installed CLI version:
```toml
[dependencies]
wasm-bindgen = "0.2"  # use the appropriate version
```

---

### 3c. Missing wasm-opt

**Symptom:**
```
[WARNING]: wasm-opt not found; not optimizing wasm
```

**Fix:**

Install `binaryen` which includes `wasm-opt`:

On Ubuntu/Debian:
```bash
sudo apt-get install binaryen
```

On macOS:
```bash
brew install binaryen
```

---

### 3d. Build Succeeds but JS Glue is Not Found

**Symptom:** Runtime error when importing the WASM module in JavaScript/TypeScript.

**Fix:**

Run `wasm-pack build` with the correct target:
```bash
# For bundlers (webpack, Vite, etc.)
wasm-pack build --target bundler

# For Node.js
wasm-pack build --target nodejs

# For browsers (no bundler)
wasm-pack build --target web
```

Output is placed in the `pkg/` directory by default. Make sure your JS imports point there.

---

### 3e. WASM Module Size Too Large

**Fix:**

Add the following to `Cargo.toml` to optimize for size:
```toml
[profile.release]
opt-level = "s"      # Optimize for size
lto = true           # Enable link-time optimization
codegen-units = 1
panic = "abort"
```

Then build in release mode:
```bash
wasm-pack build --release
```

---

## 4. React Native Integration Notes

### 4a. NDK Not Configured (Android)

**Symptom:**
```
ANDROID_NDK_HOME is not set
```

**Fix:**

1. Install the NDK via Android Studio: **SDK Manager → SDK Tools → NDK (Side by side)**
2. Set environment variables:
```bash
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/<version>    # macOS
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/<version>             # Linux
```

Add to `~/.bashrc` or `~/.zshrc` to persist.

---

### 4b. cargo-ndk Not Installed

**Symptom:**
```
error: no such subcommand: `ndk`
```

**Fix:**
```bash
cargo install cargo-ndk
```

Then build:
```bash
cargo ndk -t armeabi-v7a -t arm64-v8a -o ./jniLibs build --release
```

---

### 4c. Linking Errors on Android

**Symptom:**
```
undefined reference to `__android_log_print`
```

**Fix:**

Add the Android log library dependency in `Cargo.toml`:
```toml
[target.'cfg(target_os = "android")'.dependencies]
android_log = "0.1"
```

Or link the library manually in your `build.rs`:
```rust
println!("cargo:rustc-link-lib=log");
```

---

### 4d. iOS Build Fails with Missing lipo / Simulator Slice

**Symptom:**
```
building for iOS Simulator, but linking in object file built for iOS
```

**Fix:**

Create a universal binary using `lipo`:
```bash
lipo -create \
  target/aarch64-apple-ios/release/libmylib.a \
  target/x86_64-apple-ios/release/libmylib.a \
  -output target/universal/libmylib.a
```

Or use `cargo-lipo` for convenience:
```bash
cargo install cargo-lipo
cargo lipo --release
```

---

### 4e. Metro Bundler Doesn't Pick Up Native Changes

**Fix:**

Clear Metro cache and restart:
```bash
# For React Native CLI
npx react-native start --reset-cache

# For Expo
npx expo start --clear
```

Also make sure native module re-linking is done:
```bash
cd ios && pod install && cd ..
```

---

## 5. General Tips

- Always run `rustup update` before starting a new build to ensure your toolchain is current.
- Use `cargo check` for faster feedback without a full build.
- Run `cargo build --verbose` to get detailed output when diagnosing mysterious failures.
- Check `rust-toolchain.toml` or `rust-toolchain` file in the project root — it may pin a specific Rust version.
- When in doubt, `cargo clean` and rebuild from scratch.
- For environment variable issues, use `printenv | grep -i rust` or `printenv | grep -i android` to debug.

---

## Still Stuck?

If your issue is not covered here:
- Search existing [GitHub Issues](../../issues) — someone may have faced the same problem.
- Open a new issue with your full error log, OS, Rust version (`rustc --version`), and steps to reproduce.
- Join the community discussion for faster help.