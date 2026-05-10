// Since we're temporarily disabling the workspace configuration,
// let's also temporarily disable the protobuf generation in build.rs
// This will be re-enabled when we fix the workspace configuration

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}