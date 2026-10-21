# webpki-roots (vendored, emSign roots removed)

This is webpki-roots 1.0.10 from crates.io with three trust anchors deleted from
`TLS_SERVER_ROOTS`: emSign Root CA - C1, emSign Root CA - G1 and emSign ECC Root CA - C3.
1.0.10 itself already dropped emSign ECC Root CA - G3. Nothing else is changed; check with
`diff` against `~/.cargo/registry/src/*/webpki-roots-1.0.10/src/lib.rs`.

## Why

RUSTSEC-2026-0382: the private keys of some emSign intermediates are public, and
rustls-webpki before 0.103.16 accepts certificate paths through them. The Discord gateway
websocket goes serenity 0.12.5 -> tokio-tungstenite 0.21 -> rustls 0.22.4 -> rustls-webpki
0.102.8, which has no fixed release, so updating can't fix it. The gateway handshake carries
the bot token, so a forged `gateway.discord.gg` certificate would expose it.

With no emSign root in the trust store, no path through those intermediates can verify,
whatever the rustls-webpki version. Discord's certificates don't chain to emSign. webpki-roots
0.26.11 (which tokio-tungstenite uses) re-exports 1.x, so this one copy covers the gateway
and reqwest.

`tests/tls_roots_tests.rs` fails if the patch stops applying.

## Removing it

While this copy is patched in, `cargo update` gets no root changes from Mozilla, so drop it at
whichever of these comes first.

**Upstream drops the emSign roots.** Mozilla is tracking the distrust in
[bugzilla 2083449](https://bugzilla.mozilla.org/show_bug.cgi?id=2083449). When a webpki-roots
release no longer contains "emSign" in `src/lib.rs`:

1. Delete this directory, the `[patch.crates-io]` entry in `Cargo.toml` and the `COPY vendor`
   line in `Dockerfile`.
2. Bump the `webpki-roots` dev-dependency to that release and run
   `cargo update -p webpki-roots`.
3. Keep `tests/tls_roots_tests.rs`: it now checks the upstream release instead of this copy.
4. Keep RUSTSEC-2026-0382 in `.cargo/audit.toml`, since the gateway is still on 0.102.8, but
   change its reason to "no emSign roots in upstream webpki-roots".

**The gateway gets a fixed rustls-webpki.** Once `cargo tree -i rustls-webpki` shows only
>=0.103.16 (expected with serenity 0.13), the verifier rejects the leaked intermediates itself.
Remove everything: this directory, the patch entry, the `COPY vendor` line,
`tests/tls_roots_tests.rs`, the `webpki-roots` dev-dependency and the RUSTSEC-2026-0382 entry.
