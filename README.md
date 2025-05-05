# Mycelium Monorepo

<!-- This monorepo contains the Mycelium project, a decentralized P2P social network in which every interaction is recorded as a locally replicated, signed acknowledgement. Those pairwise histories weave together into a shared, tamper-resistant record, and users earn tokens by contributing to that record. -->

## Repository Layout

```plain
mycelium/
├─ proto/
│   └─ mycelium.proto
├─ core/
│   ├─ Cargo.toml         # package.name = "mycelium-core"
│   ├─ build.rs           # prost-build configuration
│   └─ src/
│       ├─ lib.rs         # public API exports
│       ├─ error.rs       # error definitions
│       ├─ config.rs      # configuration loader
│       ├─ ffi.rs         # FFI entrypoints (node_start, connected_peers, connect_to_peer, publish_message, subscribe_topic, send_reaction)
│       ├─ network/       # P2P networking modules
│       ├─ domain/        # SNS business logic
│       ├─ storage/       # persistence layer
│       └─ util/          # utilities
└─ ui/
    ├─ pubspec.yaml       # name: mycelium_ui
    ├─ tool/
    │   └─ gen_proto.dart  # Dart protobuf generation script
    └─ lib/
        ├─ src/
        │   ├─ mycelium_ffi.dart  # FFI bindings
        │   └─ main.dart          # Flutter entrypoint
        └─ generated/
            └─ proto/             # protoc-generated Dart code
```

## Build Instructions

### Prerequisites

* Rust toolchain (>= 1.58)
* Protocol Buffer Compiler (`protoc`)
* Dart SDK (>= 2.12)
* Flutter SDK (>= 3.0)

### 1. Generate Protobuf Code

#### Rust (core)

```bash
cd core
cargo build
```

#### Flutter (ui)

```bash
cd ui
dart tool/gen_proto.dart
```

### 2. Build Rust Core

```bash
cd core
cargo build
cargo build --release
```

### 3. Run Flutter Client

```bash
cd ui
flutter pub get
flutter run
```

## Exposed FFI Functions

* `node_start()` → initializes and runs the P2P node
* `connected_peers()` → returns a JSON list of current peers
* `connect_to_peer(address)` → dials the given peer address
* `publish_message(topic, payload)` → publishes a message to a topic
* `subscribe_topic(topic)` → subscribes to a topic
* `send_reaction(peer, message_id, reaction)` → sends a reaction to a message

## Protobuf Definition

See `proto/mycelium.proto` for the complete message definitions.
