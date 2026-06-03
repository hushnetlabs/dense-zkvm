# DenseZK SDK

A Rust SDK for the DenseZK graph-native zero-knowledge virtual machine. Generates Groth16 proofs locally using the arkworks ecosystem. Ships with native Rust and React Native (WASM) targets.

## Contact
+ [mail](contact@hushnetlabs)
+ [slack](slack.com)

## Overview

DenseZK enables privacy-preserving decentralized social networking. A user's device generates a cryptographic witness locally, then produces a zk-SNARK proof -- all without exposing private graph data.

This SDK implements a local proving pipeline:

1. **Client** -- computes a Poseidon commitment over a social graph edge, producing a lightweight witness
2. **Local Prover** -- takes the witness and public inputs, runs Groth16 setup and proving in-process, and verifies the proof before returning it

## Architecture

```
+--------------------------------------------------+
|                    Application                     |
|                                                    |
|  DenseClient          LocalProver                  |
|  +-------------+      +-------------------+        |
|  | create_     |      | setup()           |        |
|  | witness()   |----->| prove()           |        |
|  | (Poseidon)  |      | verify()          |        |
|  +-------------+      | serialize()       |        |
|                       +-------------------+        |
+--------------------------------------------------+
```

The SDK compiles to two targets:

| Target | Use case | Build command |
|---|---|---|
| Native (x86_64/aarch64) | Server-side, CLI tools, desktop apps | `cargo build --release` |
| WASM (`wasm32-unknown-unknown`) | React Native, web browsers | `cargo build --target wasm32-unknown-unknown --release` |

## Quick Start (Rust)

Add to your `Cargo.toml`:

```toml
[dependencies]
densezk-sdk = { path = "../denseZK" }
```

Minimal example:

```rust
use densezk_sdk::execute_dense_zk_flow;

fn main() {
    let result = execute_dense_zk_flow(
        456,                          // sender_id
        789,                          // receiver_id
        1,                            // edge_weight
        "0xabc123",                  // current graph root state
        1,                            // threshold
    );

    match result {
        Ok(proof) => println!("Proof generated, size: {} bytes", proof.proof_bytes.len()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Quick Start (React Native)

Install the WASM bindings:

```bash
npm install @densezk/react-native
```

Or build from source:

```bash
cd react-native-sdk
npm install
npm run build:wasm
npm run build
```

Usage:

```typescript
import { executeFlow, init, Client, Prover } from '@densezk/react-native';

await init();

// One-shot
const proof = await executeFlow(456, 789, 1, '0xabc123', 1);
console.log(`Proof size: ${proof.proof_bytes.length} bytes`);

// Or step-by-step
const client = new Client();
const witness = client.createWitness(456, 789, 1);

const prover = new Prover();
const proof = prover.prove(witness, { graph_root: '0xabc123', threshold: 1 });
```

## Manual Usage (Rust)

For fine-grained control, use the client and prover separately:

```rust
use densezk_sdk::{DenseClient, LocalProver, PublicInputs};

// Step 1: Generate witness on-device
let mut client = DenseClient::new();
let witness = client.create_witness(sender_id, receiver_id, weight)?;

// Step 2: Define public inputs
let public_inputs = PublicInputs {
    graph_root: "0x...".to_string(),
    threshold: 1,
};

// Step 3: Run local proving
let prover = LocalProver::setup()?;
let proof = prover.prove(witness, public_inputs)?;
```

## Modules

| Module | Purpose |
|---|---|
| `client` | On-device witness generation with Poseidon hashing |
| `prover` | Local Groth16 proving with arkworks |
| `crypto` | Poseidon hash over BN254 scalar field |
| `rel1cs` | Rel1CS data types (graph edges, public inputs) |
| `error` | Unified error type for all SDK operations |
| `wasm` | WASM bindings for React Native / web (wasm32 only) |

## Dependencies

- **ark-ff / ark-ec / ark-bn254** -- finite field and elliptic curve arithmetic
- **ark-groth16 / ark-relations / ark-snark** -- Groth16 zk-SNARK proving
- **light-poseidon** -- Poseidon hash implementation compatible with Circom
- **serde / serde_json** -- serialization for witness and proof payloads
- **wasm-bindgen / serde-wasm-bindgen** -- WASM bindings for JavaScript interop
- **console_error_panic_hook** -- Rust panic -> JS console error mapping
- **thiserror** -- ergonomic error definitions

## Building

### Native binary

```bash
cargo build
cargo build --release
```

Running the example binary:

```bash
cargo run
```

Expected output:

```
[Client] Witness generated with Poseidon commitment: 3227429301273914876261610954147013817301286893576706611663322465376918135905
[Prover] Proof generated locally, size: 128 bytes
Proof generated locally, size: 128 bytes
```

### WASM module

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### WASM with JS bindings (requires wasm-pack)

```bash
cargo install wasm-pack
wasm-pack build . --target bundler --release --out-dir react-native-sdk/pkg
```

### Stress test

```bash
cargo run --bin stress --release
```

## Proof Format

The `ZKProof` struct contains:

- `proof_bytes` -- compressed Groth16 proof (128 bytes for BN254), serialized with `CanonicalSerialize`
- `root_commitment` -- the graph root string passed as public input

The proof is verified internally before being returned. If verification fails, `ConstraintViolation` is returned.

## Cryptographic Details

- **Curve**: BN254 (Barreto-Naehrig)
- **Proof system**: Groth16 with circuit-specific trusted setup
- **Hash**: Poseidon with 3 inputs, x^5 S-box, BN254 scalar field, Circom-compatible parameters
- **Serialization**: Compressed canonical serialization via ark-serialize

## License

See LICENSE file.
