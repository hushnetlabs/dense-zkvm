# Rel1CS: The Relation-1 Constraint System

> **This document is the primary reference for DenseZK's core innovation.** If you're new to
> zero-knowledge proofs entirely, read the [ZK Primer](#appendix-zk-primer-for-the-curious)
> at the bottom first. If you know what R1CS is but not Rel1CS, start at
> [The Problem](#1-the-problem-graph-operations-in-r1cs) below.

---

## Table of Contents

1. [The Problem: Graph Operations in R1CS](#1-the-problem-graph-operations-in-r1cs)
2. [The Solution: Rel1CS](#2-the-solution-rel1cs)
3. [R1CS vs Rel1CS: A Comparison](#3-r1cs-vs-rel1cs-a-comparison)
4. [Security Properties](#4-security-properties)
5. [Opcode Reference](#5-opcode-reference)
   - [EDGE_MEM](#edge_mem)
   - [COUNT](#count)
   - [PATH](#path)
   - [NULLIFY](#nullify)
   - [COMMIT](#commit)
   - [VERIFY_SIG](#verify_sig)
6. [Circuit Compilation](#6-circuit-compilation)
7. [Tutorial 1 — Follower Threshold Proof](#tutorial-1--follower-threshold-proof)
8. [Tutorial 2 — Content Authorship](#tutorial-2--content-authorship)
9. [Tutorial 3 — Social Distance](#tutorial-3--social-distance)
10. [Appendix: ZK Primer for the Curious](#appendix-zk-primer-for-the-curious)

---

## 1. The Problem: Graph Operations in R1CS

Traditional zero-knowledge proof systems (Groth16, PLONK, etc.) are built on **R1CS**
(Rank-1 Constraint Systems). An R1CS circuit is essentially a collection of multiplication
gates over a finite field. Every statement you want to prove gets compiled down to a set of
equations of the form:

```
(a · x) * (b · y) = (c · z)
```

This works beautifully for arithmetic — hashing a value, checking a signature, proving
knowledge of a preimage. The problem arises when your domain is a **social graph**.

Consider this question: *"Does Alice follow Bob?"*

In a social graph, `FOLLOWS(alice, bob)` is an **edge membership** assertion. To prove it
in classical R1CS, you need to encode the graph's Merkle tree and open a path from the
root to the leaf containing the `(alice, bob)` edge. Each Merkle path step requires ~3–5
multiplication gates per hash round. For a Poseidon-based tree of depth `d`, that's
roughly `5d` constraints **per edge**.

Now consider a realistic social predicate:

> *"I have at least 100 followers, and I authored this post, and the post's author
> is within 2 degrees of Bob."*

You're looking at:

- **100 edge membership proofs** for the follower set → ~1,500+ constraints
- **1 content commitment** → ~50 constraints
- **A path traversal of depth ≤ 2** → variable, potentially hundreds more

At mobile-scale, this is prohibitive. Groth16 proving time on a phone for a circuit with
tens of thousands of constraints is measured in seconds, not milliseconds. Users notice.

The deeper issue is architectural: R1CS was not designed for **relational** data. It treats
graph edges as opaque bit-strings and encodes membership via cryptographic hashes, which
are expensive. Every social primitive (follow, like, comment, block) pays the same
overhead regardless of its semantic meaning.

---

## 2. The Solution: Rel1CS

**Rel1CS** (Relation-1 Constraint System) is DenseZK's answer to this problem. Instead of
encoding graph structure into arithmetic constraints, Rel1CS treats the graph as a
**first-class object** at the constraint-system level.

The key insight is this:

> **An edge `(u, v)` in a graph is already a binary relation. You don't need to compile
> it into field arithmetic — you can assert it directly.**

Rel1CS replaces arithmetic gates for graph operations with **graph-edge membership
assertions**. These are native operations in the ISA that map to efficiently verifiable
commitments, rather than being synthesized from scratch using multiplication gates.

Concretely, Rel1CS introduces a small instruction set (described in
[Section 5](#5-opcode-reference)) where each opcode corresponds to a common social graph
operation. The circuit compiler knows the semantics of these operations and can produce
tightly optimized Groth16 sub-circuits for each one, reusing them across predicates.

### The Intuition

Imagine you're a bank teller verifying that someone's name appears on a list. In R1CS,
you'd hash their name and compare it against a Merkle root — essentially building a
mini-hash-chain from scratch each time. In Rel1CS, you hand the teller a
**pre-certified roster**: they open the binding once (at setup time), and individual
membership checks become simple index lookups against that roster's commitment.

The "roster" is the graph's Merkle accumulator, computed over all edges using Poseidon.
EDGE_MEM assertions open paths in that accumulator, but because the accumulator's
structure is known at circuit-compile time, the verifier sub-circuit is vastly smaller.

### What Stays the Same

Rel1CS is **not** a new proof system. It compiles down to standard Groth16 over BN254,
the same as vanilla R1CS. Existing verification infrastructure — on-chain verifiers,
proof aggregation, trusted setup ceremonies — all work unchanged. Rel1CS is a
**circuit-description language** layered above R1CS, not a replacement for it.

---

## 3. R1CS vs Rel1CS: A Comparison

The table below shows typical constraint counts for common social graph predicates.
"Constraints" means R1CS multiplication gates in the final compiled circuit.

| Predicate | Naive R1CS | Rel1CS | Reduction |
|---|---|---|---|
| Single edge membership (depth-20 tree) | ~100 constraints | ~12 constraints | ~88% |
| Follower count ≥ k (k=100) | ~10,000 constraints | ~220 constraints | ~98% |
| Content commitment (Poseidon, 3 inputs) | ~50 constraints | ~50 constraints | 0% (baseline) |
| EdDSA signature verification | ~1,500 constraints | ~1,500 constraints | 0% (baseline) |
| Social distance ≤ 2 (BFS depth 2) | ~5,000+ constraints | ~200 constraints | ~96% |
| Combined follower + authorship + distance | ~16,500+ constraints | ~500 constraints | ~97% |

> **Note on baselines:** Rel1CS does not reduce constraints for pure arithmetic operations
> (hashing, signatures). Its gains are concentrated in **relational** operations — membership,
> counting, and traversal. The combined predicate row is where the savings compound.
>
> **Note on COUNT estimates:** The `Follower count ≥ k` row uses the same compiled-cost
> estimate given later in the COUNT opcode section: approximately `2k + 20` constraints.
> For `k = 100`, that is `220` constraints.

### What's happening under the hood

For `COUNT ≥ k`, naive R1CS must individually prove membership for each of `k` edges,
paying the full Merkle-opening cost per edge. A **planned/future optimization** for Rel1CS
is to use a set accumulator that commits to the entire edge set in a single polynomial,
allowing the COUNT assertion to approach approximately `O(1)` constraint cost rather than
`O(k)`. **The current implementation is still linear in `k`** (as described in the COUNT
opcode section below).

For `PATH ≤ d`, naive R1CS must enumerate candidate paths up to depth `d` and check
each, leading to exponential blowup in the worst case. In the current Rel1CS model,
`PATH` is compiled into up to `d` sequential `EDGE_MEM`-style sub-circuits, one per hop,
with witness values chaining the destination of step `i` to the source of step `i+1`.
This makes the cost scale with the allowed path depth rather than with the number of
possible paths. A more aggressive **graph adjacency commitment** checked with a single MSM
is a possible optimized formulation, but it is not the mechanism assumed by the opcode
description in this document.

---

## 4. Security Properties

### Soundness

Rel1CS inherits the soundness of Groth16. A proof is valid only if:

1. The prover knows a valid opening to the graph root commitment (i.e., the edge actually
   exists in the committed graph).
2. The committed graph root matches the one published on-chain (or passed as a public
   input).

An adversary cannot forge an edge membership proof without either breaking the Poseidon
collision resistance (computational assumption) or producing a valid Merkle opening for
an edge that wasn't committed (which requires breaking the binding of the accumulator
over BN254's scalar field).

### Zero-Knowledge

The proof reveals **nothing** about the graph structure beyond what is asserted. For
example:

- A follower-threshold proof (`COUNT ≥ 100`) reveals that the prover has at least 100
  followers, but does not reveal who those followers are.
- A social-distance proof (`PATH ≤ 2`) reveals that a path of length ≤ 2 exists, but
  does not reveal the intermediate nodes.
- A content authorship proof reveals that the prover signed the content, but reveals
  nothing about the prover's other social connections.

This is achieved by keeping all edge identifiers and intermediate nodes as **private
witnesses** in the Groth16 proof, with only the final assertion (threshold met, distance
satisfied, etc.) surfaced as a public input.

### Sybil Resistance

The NULLIFY opcode computes a **nullifier** — a deterministic, unlinkable commitment to
a (user, context) pair. This prevents a single user from submitting the same proof
multiple times under different identities. See [NULLIFY](#nullify) for details.

---

## 5. Opcode Reference

Each opcode in the Rel1CS ISA compiles to a Groth16 sub-circuit. Constraint counts are
for the BN254 curve with a Poseidon hash function (width 3, x^5 S-box).

---

### `EDGE_MEM`

**Assert edge membership via Merkle opening.**

The most fundamental opcode. Proves that a directed edge `(u, v, w)` — where `u` is the
source node, `v` is the target node, and `w` is an optional weight — exists in the
committed graph at the current root.

#### Syntax

```
EDGE_MEM(src: NodeId, dst: NodeId, weight: Field, root: Root) -> Bool
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `src` | `NodeId` (private) | Source node identifier |
| `dst` | `NodeId` (private) | Destination node identifier |
| `weight` | `Field` (private) | Edge weight (use `1` for unweighted graphs) |
| `root` | `Root` (public) | Merkle root of the committed graph |

#### Constraint Count

Approximately 64 constraints for a tree of depth 20 (supporting up to ~1M edges), using a 2-to-1 Poseidon compression function at each level.

For a tree of depth d, the count is ⌈d / 2⌉ × 6 + 4 constraints.

#### Example Usage

```
// Prove that user 456 follows user 789
EDGE_MEM(
    src    = 456,
    dst    = 789,
    weight = 1,
    root   = "0xabc123..."   // current graph root, published on-chain
)
```

In the Rust SDK, this corresponds to:

```rust
let witness = client.create_witness(
    456,        // sender_id (src)
    789,        // receiver_id (dst)
    1,          // edge_weight
)?;
```

---

### `COUNT`

**Assert that a node's edge set has cardinality ≥ k.**

Proves that a node has at least `k` outgoing (or incoming) edges in the committed graph,
without revealing which edges those are.

#### Syntax

```
COUNT(node: NodeId, direction: {IN | OUT}, k: u64, root: Root) -> Bool
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `node` | `NodeId` (private) | The node whose edges are being counted |
| `direction` | `IN` or `OUT` (public) | Count incoming or outgoing edges |
| `k` | `u64` (public) | Minimum cardinality threshold |
| `root` | `Root` (public) | Merkle root of the committed graph |

#### Constraint Count

Approximately **`2k + 20` constraints** in the current implementation. This is linear in
`k` in the worst case, but typical mobile predicates use `k ≤ 1000`, keeping circuit
size manageable. A polynomial accumulator optimization (reducing to `O(log k)`) is
planned for a future release.

#### Example Usage

```
// Prove that I have at least 100 followers
COUNT(
    node      = MY_USER_ID,
    direction = IN,
    k         = 100,
    root      = CURRENT_GRAPH_ROOT
)
```

---

### `PATH`

**Assert that the social distance between two nodes is ≤ d.**

Proves that there exists a path of length at most `d` between `src` and `dst` in the
graph, without revealing the intermediate nodes along the path.

#### Syntax

```
PATH(src: NodeId, dst: NodeId, max_depth: u8, root: Root) -> Bool
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `src` | `NodeId` (private) | Start node |
| `dst` | `NodeId` (public or private) | End node |
| `max_depth` | `u8` (public) | Maximum allowed path length |
| `root` | `Root` (public) | Merkle root of the committed graph |

#### Constraint Count

Approximately **`12 × max_depth` constraints** for the `EDGE_MEM` checks themselves,
plus a small linear overhead to link consecutive hops and gate unused slots. At
depth 2 (the most common case for social applications), this is roughly **24
constraints for membership checks**, or the **mid-20s overall** including the path
chaining logic.

The PATH opcode represents the intermediate nodes as private witnesses and verifies
each hop using an `EDGE_MEM` sub-circuit, so its cost scales linearly with the
number of hop slots. The depth bound `max_depth` is enforced structurally — the
circuit has exactly `max_depth` `EDGE_MEM` slots, and unused slots are gated so they
do not contribute to the path while still satisfying the fixed circuit shape.

#### Example Usage

```
// Prove that I am within 2 degrees of user 42
PATH(
    src       = MY_USER_ID,
    dst       = 42,
    max_depth = 2,
    root      = CURRENT_GRAPH_ROOT
)
```

#### ASCII Diagram

```
MY_USER_ID ---[hop 1]---> INTERMEDIATE_NODE ---[hop 2]---> USER_42
               ↑                                 ↑
               EDGE_MEM assertion                EDGE_MEM assertion
               (private witness)                 (private witness)
```

---

### `NULLIFY`

**Compute a nullifier for Sybil resistance.**

Derives a deterministic, context-bound nullifier from the user's identity and a
domain-separation tag. Publishing the nullifier on-chain prevents double-use of a proof
(e.g., voting twice, claiming a reward twice) without revealing the user's identity.

#### Syntax

```
NULLIFY(identity: Secret, context: Field) -> Nullifier
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `identity` | `Secret` (private) | User's secret key or derived identity scalar |
| `context` | `Field` (public) | Domain separator (e.g., campaign ID, block number) |

#### Output

| Field | Description |
|---|---|
| `nullifier` | `Field` (public) — published on-chain to prevent double-use |

#### Constraint Count

Approximately **50 constraints** (a single Poseidon evaluation over 2 inputs).

#### How Nullifiers Work

```
nullifier = Poseidon(identity, context)
```

The `context` binds the nullifier to a specific action or campaign. The same user
will produce the same nullifier for the same context (preventing double-spend) but
different nullifiers for different contexts (preserving unlinkability across actions).

An on-chain registry stores published nullifiers. Before accepting a proof, the
verifier checks that the nullifier has not been seen before.

#### Example Usage

```
// Generate a nullifier for a follower-gated campaign with ID 7
NULLIFY(
    identity = MY_SECRET,
    context  = 7          // campaign ID
)
```

---

### `COMMIT`

**Compute a content commitment.**

Produces a Poseidon commitment over a content payload. Used to bind a proof to a
specific piece of content (a post, an image hash, a vote) without revealing the content
itself — or, alternatively, to reveal the commitment publicly so others can verify
authorship.

#### Syntax

```
COMMIT(payload: [Field; N]) -> Commitment
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `payload` | `[Field; N]` (private or public) | Content fields to commit to |

The payload is typically the content's hash split into field elements. For content of
arbitrary length, hash it with Poseidon externally first, then pass the
digest as the payload.

#### Output

| Field | Description |
|---|---|
| `commitment` | `Field` (public) — the Poseidon digest of the payload |

#### Constraint Count

Approximately **50 constraints** per 3-field block (one Poseidon permutation). For a
single 256-bit digest split into 8 field elements: ~3 Poseidon calls → ~150 constraints.

#### Example Usage

```
// Commit to a post with content hash 0xdeadbeef...
COMMIT(payload = [post_hash_lo, post_hash_hi])
// → commitment = Poseidon(post_hash_lo, post_hash_hi)
```

---

### `VERIFY_SIG`

**Verify an EdDSA signature.**

Proves that the prover holds a private key corresponding to a known public key, and that
they used it to sign a specific message (or commitment). DenseZK uses the Baby JubJub
twisted Edwards curve, which is BN254-native and significantly cheaper to verify in a
Groth16 circuit than secp256k1.

#### Syntax

```
VERIFY_SIG(
    pubkey:    PublicKey,
    message:   Field,
    signature: (Field, Field)
) -> Bool
```

#### Parameters

| Parameter | Type | Description |
|---|---|---|
| `pubkey` | `PublicKey` (public) | The signer's Baby JubJub public key |
| `message` | `Field` (public) | The message or commitment that was signed |
| `signature` | `(Field, Field)` (private) | The `(R, s)` EdDSA signature pair |

#### Constraint Count

Approximately **1,500 constraints** — this is the most expensive opcode. Baby JubJub
signature verification requires a scalar multiplication on a twisted Edwards curve,
which is the fundamental cost floor for any signature scheme inside a SNARK.

#### Example Usage

```
// Prove that I (pubkey MY_PUBKEY) signed commitment C
VERIFY_SIG(
    pubkey    = MY_PUBKEY,
    message   = C,         // e.g., the output of a COMMIT call
    signature = (R, s)     // my EdDSA signature over C
)
```

---

## 6. Circuit Compilation

A Rel1CS predicate is a composition of opcodes. This section describes the intended
compiler pipeline at a conceptual level. It is **not** a description of a currently
available `densezk` CLI or of directories that exist in this repository today.

In the intended flow, a compiler would turn a Rel1CS predicate into a concrete
Groth16-compatible circuit, then run setup to produce proving and verification keys.
The file names, commands, and directories below are illustrative placeholders for that
future workflow.

### Step 1 — Define the Predicate

Predicates are expressed in the Rel1CS DSL (see [Tutorial 1](#tutorial-1--follower-threshold-proof)
for a concrete example). In a future implementation, a predicate could be stored as a
source file such as `my_predicate.rel`.

### Step 2 — Compile to a Constraint System

Conceptually, the compiler would:
1. Parse the predicate source and resolve opcode calls.
2. Expand each opcode into its corresponding R1CS sub-circuit.
3. Wire sub-circuits together, threading public inputs and witnesses.
4. Emit a circuit description in a serialized format such as JSON or ASN.1.

**Illustrative circuit JSON format (excerpt):**
```json
{
  "name": "follower_threshold",
  "curve": "bn254",
  "num_public_inputs": 2,
  "num_constraints": 420,
  "opcodes": [
    { "op": "COUNT", "params": { "direction": "IN", "k": 100 } },
    { "op": "NULLIFY", "params": { "context": "campaign_7" } }
  ]
}
```

**ASN.1 binary format** is used for production deployments where circuit size matters
(mobile downloads). The `.asn1` file is typically 10–20× smaller than the JSON
equivalent.

### Step 3 — Trusted Setup

Groth16 requires a circuit-specific **trusted setup** — a one-time ceremony that
produces the proving key (`pk`) and verification key (`vk`).

```
densezk setup circuits/my_predicate.json \
    --ptau tau.ptau \
    --pk   keys/my_predicate.pk \
    --vk   keys/my_predicate.vk
```

The `tau.ptau` file is a Powers of Tau file — a public, reusable ceremony artifact. The
repo ships `tau.ptau` at the root, which supports circuits up to 2^20 constraints. For
larger circuits, use a tau file from a larger ceremony (Hermez Phase 1 transcripts are
publicly available).

The proving key `pk` stays on the device (it's large, ~10–50 MB). The verification key
`vk` is tiny (~1 KB) and is posted on-chain or shared with verifiers.

### Step 4 — Prove and Verify

```rust
use densezk_sdk::{DenseClient, LocalProver, PublicInputs};

// Witness generation (on-device, private)
let witness = client.create_witness(sender_id, receiver_id, weight)?;

// Public inputs (known to both prover and verifier)
let public_inputs = PublicInputs {
    graph_root: "0xabc123...".to_string(),
    threshold: 100,
};

// Proving (on-device, private)
let prover = LocalProver::setup()?;
let proof  = prover.prove(witness, public_inputs)?;
// proof.proof_bytes is 128 bytes (compressed Groth16 on BN254)
```

### Circuit Output Summary

| Artifact | Size | Who holds it |
|---|---|---|
| Circuit JSON (`.json`) | ~5–50 KB | Open source / public |
| Circuit ASN.1 (`.asn1`) | ~0.5–5 KB | Bundled in app |
| Proving key (`.pk`) | ~10–50 MB | User's device |
| Verification key (`.vk`) | ~1 KB | On-chain / verifier |
| Proof (`.proof`) | 128 bytes | Published by prover |

---

## Tutorial 1 — Follower Threshold Proof

**Goal:** Prove that you have at least 100 followers, without revealing who they are.

### What you'll need

- Your user ID (private)
- The current graph Merkle root (public, fetched from chain)
- A nullifier context (public) to prevent double-use of the proof
- The DenseZK SDK installed

### The Predicate

```
// predicates/follower_threshold.rel

predicate FollowerThreshold(
    // Public inputs
    graph_root : Root,
    threshold  : u64,
    context    : Field,
    nullifier  : Field,

    // Private witness
    user_id    : NodeId,
    identity   : Secret,
) {
    // Assert: I have at least `threshold` followers
    COUNT(
        node      = user_id,
        direction = IN,
        k         = threshold,
        root      = graph_root,
    );

    // Assert: this nullifier was derived from my identity
    // (prevents submitting the same proof twice)
    assert nullifier == NULLIFY(
        identity = identity,
        context  = context,
    );
}
```

### Rel1CS Constraints

For `threshold = 100`:

| Opcode | Constraints |
|---|---|
| `COUNT(IN, k=100)` | 220 |
| `NULLIFY` | 50 |
| **Total** | **270** |

For comparison, a naive R1CS encoding of 100 EDGE_MEM calls would cost ~10,000
constraints.

### Witness Structure

```rust
pub struct FollowerThresholdWitness {
    // Private (never leaves the device)
    pub user_id:        u64,
    pub identity:       [u8; 32],   // secret scalar on Baby JubJub
    pub follower_edges: Vec<Edge>,  // the actual follower edges, as Merkle witnesses

    // Public inputs (shared with verifier)
    pub graph_root:     String,     // "0xabc..."
    pub threshold:      u64,        // 100
    pub context:        u64,        // campaign ID or block number
    pub nullifier:      [u8; 32],   // Poseidon(identity, context)
}
```

### End-to-End Flow

```
    Your device                         Chain / Verifier
    ──────────────                      ────────────────
    1. Fetch graph_root ──────────────> published on-chain
    2. Enumerate local follower edges
    3. compute_witness(edges)
    4. prove(witness, public_inputs) 
    5. publish(proof, nullifier) ──────> 6. check nullifier not seen before
                                         7. verify(proof, vk, public_inputs)
                                         8. record nullifier
```

Steps 2–4 happen entirely on the user's device. The verifier learns only:
- You have ≥ 100 followers (the assertion)
- Your nullifier (to prevent double-use)

It learns nothing about the graph structure.

---

## Tutorial 2 — Content Authorship

**Goal:** Prove that you authored a specific post, identified by its content hash.

This combines `COMMIT` (to bind the proof to the content) and `VERIFY_SIG` (to prove
you hold the key that signed it).

### The Predicate

```
// predicates/authorship.rel

predicate ContentAuthorship(
    // Public inputs
    content_hash_high: Field, // Upper 128 bits of SHA-256
    content_hash_low: Field,  // Lower 128 bits of SHA-256
    author_pubkey: PublicKey,

    // Private witness
    sig_r: Field,
    sig_s: Field,
) {
    // Step 1: commit to the content using both limbs
    // This ensures no data is lost to modular reduction
    let commitment = COMMIT(payload = [content_hash_high, content_hash_low]); 
    // Step 2: verify the author's signature over the commitment
    VERIFY_SIG(
        pubkey    = author_pubkey,
        message   = commitment,
        signature = (sig_r, sig_s),
    );
}
```

### Why COMMIT before VERIFY_SIG?

You could sign `content_hash` directly. Using `COMMIT` first adds a Poseidon layer
that keeps the signature malleable only within the ZK circuit — the signature is over
a field element in BN254's scalar field, not an arbitrary byte string, which avoids
serialization edge cases and keeps the constraint count lower.

### Rel1CS Constraints

| Opcode | Constraints |
|---|---|
| `COMMIT` (1 field → 1 Poseidon) | 50 |
| `VERIFY_SIG` (Baby JubJub EdDSA) | 1,500 |
| **Total** | **1,550** |

This is similar to what you'd pay in vanilla R1CS — signature verification is
inherently arithmetic and Rel1CS doesn't reduce it. The win here is that DenseZK uses
the Baby JubJub curve, which is ~2× cheaper than secp256k1 for in-circuit verification.

### Witness Structure

```rust
pub struct AuthorshipWitness {
    // Public
    pub content_hash:  [u8; 32],
    pub author_pubkey: BabyJubJubPoint,

    // Private
    pub sig_r:         [u8; 32],
    pub sig_s:         [u8; 32],
}
```

### Putting It Together in Rust

```rust
use densezk_sdk::{DenseClient, LocalProver, PublicInputs};

// Off-circuit: sign the content hash with your EdDSA key
let commitment    = poseidon_hash(&[content_hash_field]);
let (sig_r, sig_s) = eddsa_sign(&my_secret_key, commitment);

// Build the witness
let witness = AuthorshipWitness {
    content_hash:  post.poseidon_digest(),
    author_pubkey: my_public_key,
    sig_r,
    sig_s,
};

// Prove
let proof = prover.prove_authorship(witness)?;
```

---

## Tutorial 3 — Social Distance

**Goal:** Prove that you are connected to a target user within 2 hops, without
revealing the intermediate node.

### The Predicate

```
// predicates/social_distance.rel

predicate SocialDistance(
    // Public inputs
    graph_root  : Root,
    target      : NodeId,   // the user you claim to be connected to
    max_depth   : u8,       // 2

    // Private witness
    src         : NodeId,   // your user ID
) {
    PATH(
        src       = src,
        dst       = target,
        max_depth = max_depth,
        root      = graph_root,
    );
}
```

### What happens internally

The `PATH` opcode expands into `max_depth` sequential `EDGE_MEM` sub-circuits, one per
hop. For `max_depth = 2`:

```
            ┌──────────────────────────────────────────────────────┐
            │  PATH(src, dst, max_depth=2) expands to:             │
            │                                                       │
            │  witness: hop1_node (private)                         │
            │                                                       │
            │  EDGE_MEM(src,      hop1_node, 1, root)  ← hop 1     │
            │  EDGE_MEM(hop1_node, dst,      1, root)  ← hop 2     │
            └──────────────────────────────────────────────────────┘
```

The intermediate node `hop1_node` is a private witness. The verifier learns only that
*some* intermediate node exists, not who it is.

### Rel1CS Constraints

| Opcode | Constraints |
|---|---|
| `PATH(max_depth=2)` | ~200 (2 × EDGE_MEM + bookkeeping) |
| **Total** | **~200** |

### Witness Structure

```rust
pub struct SocialDistanceWitness {
    // Public
    pub graph_root: String,
    pub target:     u64,
    pub max_depth:  u8,

    // Private
    pub src:            u64,
    pub intermediate:   Vec<u64>,   // length = max_depth - 1
    pub hop_witnesses:  Vec<EdgeWitness>, // Merkle opening for each hop
}
```

### Example: Building the Witness

```rust
// You know that you → @alice → @target
let witness = SocialDistanceWitness {
    graph_root: current_root,
    target:     target_user_id,
    max_depth:  2,

    // Private
    src:           my_user_id,
    intermediate:  vec![alice_user_id],
    hop_witnesses: vec![
        graph.merkle_opening(my_user_id,    alice_user_id),
        graph.merkle_opening(alice_user_id, target_user_id),
    ],
};

let proof = prover.prove_distance(witness)?;
```

The `hop_witnesses` are Merkle path openings in the graph accumulator. They're computed
locally on the device and are never transmitted.

---

## Appendix: ZK Primer for the Curious

If you've never worked with zero-knowledge proofs before, here's the minimum you need to
follow this document.

**What is a ZK proof?**
A zero-knowledge proof lets you convince someone that a statement is true, without
revealing *why* it's true. Classic example: you can prove you know a password without
typing it.

**What is a circuit?**
In ZK proofs, the statement you want to prove is encoded as a mathematical "circuit" —
essentially a big set of equations. The prover's job is to find values (the "witness")
that satisfy all equations. The verifier checks the equations without seeing the values.

**What is R1CS?**
Rank-1 Constraint System. A specific way of writing circuit equations: each equation
looks like `(a₁ · w₁ + a₂ · w₂ + ...) × (b₁ · w₁ + ...) = (c₁ · w₁ + ...)`. Modern
ZK proof systems (Groth16, PLONK) all work with R1CS or something equivalent.

**What is Groth16?**
A specific, well-studied ZK proof system. Given an R1CS circuit, Groth16 produces a
128-byte proof that can be verified in a few milliseconds. It requires a one-time
trusted setup per circuit. DenseZK uses Groth16 over the BN254 elliptic curve.

**What is Merkle tree / Merkle opening?**
A Merkle tree is a way of committing to a set of values (like a list of edges) using a
single hash (the "root"). A Merkle opening is a proof that a specific value is in the
set, consisting of the sibling hashes along the path from the leaf to the root. If you
know the root, and someone gives you a valid opening, you know the value is genuinely in
the committed set.

**Further reading:**
- [ZKProof Community Reference](https://zkproof.org/2020/08/12/reference/)
- [Vitalik's intro to STARKs and SNARKs](https://vitalik.eth.limo/general/2021/01/26/snarks.html)
- [arkworks documentation](https://docs.rs/ark-relations/latest/ark_relations/) (the Rust library DenseZK builds on)
- This document is the primary mathematical treatment currently included in the repo.
