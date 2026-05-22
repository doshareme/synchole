# WASM Binding

Browser builds should target Web Workers first:

- Run sync in a dedicated worker to avoid blocking the UI thread.
- Use browser-compatible SQLite such as OPFS-backed SQLite WASM.
- Prefer WebRTC data channels for direct browser peers.
- Use WebSocket signaling for discovery and NAT candidate exchange.
- Use relay fallback when WebRTC connectivity fails.
- Persist identity keys in WebCrypto-backed storage where possible.
