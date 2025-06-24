use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=./ubench.omniglot.toml");
    println!("cargo:rerun-if-changed=./c_src/ubench.c");
    println!("cargo:rerun-if-changed=./c_src/ubench.h");

    let cflags = std::env::var("OG_BINDGEN_CFLAGS").expect("Please set OG_BINDGEN_CFLAGS");

    let bindings = bindgen::Builder::default()
        .header("c_src/ubench.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .omniglot_configuration_file(Some(
            PathBuf::from("./ubench.omniglot.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libubench_bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
        .file("c_src/ubench.c")
        .compile("libubench");
}
