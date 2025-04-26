# proto/ - Mycelium Protocol Definitions

This directory contains all protocol buffer (`.proto`) definitions used in the Mycelium project.

## Structure

```txt
proto/
├── grpc/
│   ├── mycelium_service.proto # gRPC service API for Flutter <-> Rust
│   ├── copy.proto # Copy request/response messages
│   ├── echo.proto # Echo request/response messages
│   ├── search.proto # Search posts locally or remotely
│   └── common.proto # Common types (PeerId, PostId, Post struct)
├── p2p/
│   ├── message.proto # P2P action messages (Copy, Echo propagation)
│   ├── envelope.proto # Signed message envelope
│   └── search.proto # Search query and response messages via P2P
```

## Layer Overview

### 1. gRPC API (Flutter <-> Rust Node)

- **Purpose**:  
  Handle Copy, Echo, Post creation, and Search operations initiated by the mobile client.

- **Key Operations**:
  - `CopyPost`: Copy a post from a peer.
  - `EchoPost`: Echo an already copied post to others.
  - `CreatePost`: Create a new original post (supporting replies/quotes).
  - `SearchPost`: Search posts either locally or across the network.

- **Protocol Buffers**:
  - `mycelium_service.proto`
  - `copy.proto`
  - `echo.proto`
  - `search.proto`
  - `common.proto`

---

### 2. P2P Protocol (Rust Node <-> Rust Node)

- **Purpose**:  
  Propagate user actions and search requests across the P2P network.

- **Key Messages**:
  - `ActionMessage`: Represents a Copy or Echo action on a post.
  - `Envelope`: Contains a signed ActionMessage for authenticity.
  - `SearchQueryMessage`: A search keyword propagation message.
  - `SearchResultMessage`: A search result return message.

- **Protocol Buffers**:
  - `message.proto`
  - `envelope.proto`
  - `search.proto`

---

## Data Models

### Post Structure

The `Post` message structure now includes:

- `post_id`: Unique identifier for the post
- `parent_post_id`: Optional reference to a parent post (for reply/quote)
- `content`: Body text of the post
- `author`: Peer ID of the original author
- `created_at`: Timestamp of creation

---

## Notes

- These protocol definitions are **finalized for the PoC phase**.
- Breaking changes may still occur during active development.
- Full stabilization will precede the first public release.

---

(c) mistwood-labs
