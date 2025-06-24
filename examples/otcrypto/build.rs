use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./otcrypto.omniglot.toml");
    println!("cargo:rerun-if-changed=./c_src/omniglot_otcrypto_tbf.h");

    let cflags = std::env::var("OG_BINDGEN_CFLAGS").expect("Please set OG_BINDGEN_CFLAGS");
    // panic!("CFLAGS: {}", cflags);
    // panic!("CFLAGS: {:?}", cflags.split(" ").collect::<Vec<_>>());

    let bindings = bindgen::Builder::default()
        .header("c_src/omniglot_otcrypto_tbf.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .omniglot_configuration_file(Some(
            PathBuf::from("./otcrypto.omniglot.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libotcrypto_bindings.rs"))
        .expect("Couldn't write bindings!");

    println!(
        "cargo::rustc-link-search={}/../../third-party/opentitan-cryptolib",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
    );
    println!("cargo::rustc-link-lib=otcrypto");
}
