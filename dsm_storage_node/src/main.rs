// DSM Storage Node Library
//
// This library provides the core components for the DSM Storage Node.
// To run the storage node server, use the binary target:
//
//   cargo run --bin storage_node
//
// This file exists to maintain the library structure while
// the actual server implementation is in src/bin/storage_node.rs

fn main() {
    eprintln!("This is a library crate. To run the storage node server, use:");
    eprintln!("  cargo run --bin storage_node");
    std::process::exit(1);
}
