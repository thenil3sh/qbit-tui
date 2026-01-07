# Test Examples for qbit Project

This document outlines various test examples for the `qbit` project, categorized into Unit, Integration, and CLI tests. These examples aim to provide a starting point for ensuring the quality and correctness of your application.

## 1. Unit Tests

Unit tests focus on individual components (functions, methods) in isolation. In Rust, these are often placed in a `tests` module within the same file as the code being tested.

### Example for `src/torrent/metadata.rs`:

If you have a function that parses a `.torrent` file, you'd want to test it with:

*   A valid `.torrent` file to ensure correct parsing of all fields.
*   A malformed or corrupted `.torrent` file to ensure it handles errors gracefully and reports them appropriately.
*   A `.torrent` file with missing but optional fields to verify default handling or correct omission.
*   A `.torrent` file with extra, unexpected fields to check for robustness against unknown data.

### Example for `src/bin/bencode.rs`:

For Bencode decoding logic, you could write tests to:

*   Decode a bencoded string (e.g., `4:spam`).
*   Decode a bencoded integer (including positive `i3e`, negative `i-3e`, and zero `i0e`).
*   Decode a bencoded list (e.g., `l4:spam4:eggse`).
*   Decode a bencoded dictionary (e.g., `d3:cow3:moo4:spam4:eggse`).
*   Test that your decoder correctly fails on invalid bencoded data (e.g., `i-0e`, non-numeric characters where numbers are expected).

## 2. Integration Tests

Integration tests verify how different parts of your application work together. In Rust, these are typically placed in the `tests/` directory at the root of your project.

### Example for Tracker Communication (`src/tracker/mod.rs`):

An integration test could simulate interaction with a BitTorrent tracker:

1.  **Mock a Tracker Server:** Set up a lightweight mock HTTP/UDP server that simulates a BitTorrent tracker, capable of receiving announce requests and sending responses.
2.  **Send Request:** Use your tracker client code to send an announce request to the mock server.
3.  **Parse Response:** Assert that the client correctly parses the response received from the mock server (e.g., extracts peer information, interval, etc.).
4.  **Test Various Scenarios:** Cover different tracker responses, including successful announces, error messages from the tracker, redirects, and timeouts.

### Example for Peer Handshake (`src/peer/handshake.rs`):

This test would focus on the initial communication protocol between two peers:

1.  **Simulate Two Peers:** Create two simulated peer connections (e.g., using in-memory streams or local TCP sockets).
2.  **Initiate Handshake:** Have one simulated peer initiate the BitTorrent handshake process with the other.
3.  **Verify Success:** Assert that the handshake completes successfully, meaning both peers exchange the correct handshake messages and acknowledge each other's info hash and peer ID.
4.  **Test Failing Handshake:** Introduce scenarios where the handshake should fail, such as an incorrect info hash, an unsupported protocol, or a malformed handshake message, and verify that your code handles these errors correctly.

## 3. Command-Line Interface (CLI) Tests

Since your project has binaries in `src/bin/`, you should also test their command-line functionality. The `assert_cmd` crate is a popular choice for this in Rust.

### Example for `src/bin/tracker.rs` (if it's a CLI tool for tracker interaction):

If this binary is designed to contact a tracker given a `.torrent` file, you could test it by:

1.  **Execute Command:** Run your compiled `tracker` binary from the command line with a path to a test `.torrent` file as an argument (e.g., `cargo run --bin tracker -- test.torrent`).
2.  **Verify Exit Status:** Assert that the command exits successfully (exit code 0) if the operation was expected to succeed, or with a non-zero exit code on expected failures.
3.  **Check Standard Output/Error:** Verify the command's standard output to ensure it prints the expected information (e.g., list of peers, tracker response messages). Also, check standard error for any error messages in failure scenarios.
4.  **Test Edge Cases:**
    *   Provide a non-existent `.torrent` file path.
    *   Provide a malformed `.torrent` file.
    *   Test with valid but unusual torrent data.

These examples provide a comprehensive approach to testing your `qbit` application, covering everything from individual components to end-to-end CLI interactions.