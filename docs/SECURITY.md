# Security Model

Synchole assumes the relay, coordination, and discovery services are untrusted
for payload confidentiality. They may help peers find each other, but peer
communication is authenticated and end-to-end encrypted.

## Identity

- Every device has a stable device identity.
- Devices are enrolled into a user trust graph.
- Peer identity is validated before accepting sync frames.
- Device revocation must be checked during discovery and session setup.
- Platform key storage should be used where available:
  - Android Keystore.
  - iOS/macOS Keychain or Secure Enclave.
  - Windows DPAPI or CNG.
  - Linux Secret Service, TPM-backed storage, or app-managed key files.
  - Browser WebCrypto-backed keys where extractability can be controlled.

## Transport Encryption

Recommended production handshake:

- Long-term signing identity for authentication.
- Ephemeral X25519 key exchange for forward secrecy.
- Noise-style transcript binding.
- ChaCha20-Poly1305 or AES-GCM AEAD.
- BLAKE3 or HKDF-derived session keys.

Each encrypted frame should bind:

- sender and receiver device identities,
- session ID,
- sequence number,
- stream ID,
- packet class,
- protocol version.

## Replay Protection

- Sessions maintain monotonically increasing frame sequence numbers.
- Receivers reject old or duplicate sequence numbers inside a replay window.
- Sync object versions use version vectors and hybrid logical clocks.
- Enrollment tokens and relay reservations carry expirations.

## Relay Privacy

Relay servers forward encrypted frames only. They should not receive object keys
in plaintext unless the application explicitly accepts metadata exposure.

Operational relay controls should include:

- rate limits,
- abuse detection,
- regional policy,
- per-user quotas,
- observable connection metrics without payload logging.

## Security Tests

Security tests should cover:

- unauthorized peer rejection,
- identity mismatch rejection,
- invalid signatures,
- session downgrade attempts,
- replayed frames,
- tampered ciphertext,
- relay plaintext inspection prevention,
- device revocation.

The scaffold includes example security test files under `tests/security`.
