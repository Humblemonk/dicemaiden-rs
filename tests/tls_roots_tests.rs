// tests/tls_roots_tests.rs - Trust store guard for RUSTSEC-2026-0382
//
// The gateway's rustls-webpki 0.102.8 accepts paths through leaked emSign intermediates,
// so Cargo.toml patches webpki-roots with a copy that has no emSign roots
// (vendor/webpki-roots/README.md). If the patch stops applying, for example because a
// dependency moves to a webpki-roots version the copy doesn't satisfy, Cargo only warns;
// this test makes it fail instead.

#[test]
fn trust_store_has_no_emsign_roots() {
    let emsign: Vec<String> = webpki_roots::TLS_SERVER_ROOTS
        .iter()
        .map(|anchor| String::from_utf8_lossy(anchor.subject.as_ref()).into_owned())
        .filter(|subject| subject.contains("emSign"))
        .collect();

    assert!(
        emsign.is_empty(),
        "webpki-roots still trusts emSign roots; is the [patch.crates-io] entry applied? {emsign:?}"
    );
    assert!(
        webpki_roots::TLS_SERVER_ROOTS.len() > 100,
        "trust store looks truncated: {} roots",
        webpki_roots::TLS_SERVER_ROOTS.len()
    );
}
