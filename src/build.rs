// This build script is invoked by cargo when it is building our crate. It is responsible for
// generating code based on other protocol buffer definitions.
//
// Specifically, it generates Rust code from the `packets.proto` file in the root of our
// crate. This generated code is then compiled into our final binary.
//
// The `prost_build` crate is responsible for doing the actual work of generating code from
// protocol buffer definitions. We're passing it the path to our `.proto` file and the root
// directory of our crate.
extern crate prost_build;

fn main() {
    // Invoke the `compile_protos` function from the `prost_build` crate. This function takes
    // two arguments: a list of `.proto` files to compile and the root directory of our crate.
    // It returns a `Result` indicating whether the compilation was successful or not.
    //
    // The `.unwrap()` method is then called on the `Result` to panic if the compilation
    // failed. This is okay in a build script because it will stop the build process and
    // prevent our code from being built.
    prost_build::compile_protos(&["packets.proto"], &["."]).unwrap();
}

