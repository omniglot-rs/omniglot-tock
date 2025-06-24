use std::env;
use std::path::{Path, PathBuf};
use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=./og_littlefs.omniglot.toml");
    println!("cargo:rerun-if-changed=./c_src/og_littlefs.h");

    let cflags = std::env::var("OG_BINDGEN_CFLAGS").expect("Please set OG_BINDGEN_CFLAGS");
    // panic!("CFLAGS: {}", cflags);
    // panic!("CFLAGS: {:?}", cflags.split(" ").collect::<Vec<_>>());

    let bindings = bindgen::Builder::default()
        .header("c_src/og_littlefs.h")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .omniglot_configuration_file(Some(
            PathBuf::from("./og_littlefs.omniglot.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("og_littlefs_bindings.rs"))
        .expect("Couldn't write bindings!");

    // TODO: replace with env var once stable (https://github.com/rust-lang/cargo/issues/3946)
    let workspace_root_dir = Path::new("../..").canonicalize().unwrap();

    let littlefs_build_path = workspace_root_dir.join("third-party/littlefs/build-riscv32");
    let littlefs_built_obj = littlefs_build_path.join("lfs.o");
    if !littlefs_built_obj.exists() {
        panic!("littefs does not seem to have been built yet, ensure that {} exists", littlefs_built_obj.display());
    }
    println!(
        "cargo::rustc-link-search={}", littlefs_build_path.display(),
    );

    // We need to find the "correct" libtock-c newlib and libc++ checkouts
    // that contain the RISC-V static libraries to link with. Search for
    // an appropriate directory:
    let newlib_path =
        fs::read_dir(workspace_root_dir.join("third-party/libtock-c/lib"))
        .expect("third-party/libtock-c/lib does not exist, did you check out submodules?")
        .map(|dir_entry| dir_entry.unwrap())
        .find(|dir_entry| {
            dir_entry.file_name().to_str().unwrap().starts_with("libtock-newlib-") && dir_entry.file_type().unwrap().is_dir()
        })
        .map(|dir_entry| {
            dir_entry.path()
        })
        .expect("Could not find a downloaded newlib version to use in third-party/libtock-c/lib");

    let libcpp_path =
        fs::read_dir(workspace_root_dir.join("third-party/libtock-c/lib"))
        .expect("third-party/libtock-c/lib does not exist, did you check out submodules?")
        .map(|dir_entry| dir_entry.unwrap())
        .find(|dir_entry| {
            dir_entry.file_name().to_str().unwrap().starts_with("libtock-libc++-") && dir_entry.file_type().unwrap().is_dir()
        })
        .map(|dir_entry| {
            dir_entry.path()
        })
        .expect("Could not find a downloaded libc++ version to use in third-party/libtock-c/lib");

    let libgcc_path =
        fs::read_dir(libcpp_path.join("riscv/lib/gcc/riscv64-unknown-elf"))
        .expect(&format!("ERROR: libc++ {} does not contain libgcc", libcpp_path.display()))
        .map(|dir_entry| dir_entry.unwrap())
        .find(|dir_entry| {
            dir_entry.file_type().unwrap().is_dir()
        })
        .map(|dir_entry| {
            dir_entry.path()
        })
        .expect(&format!("Could not find a libgcc library within the selected libc++: {}", libcpp_path.display()));

    println!(
        "cargo::rustc-link-search={}/riscv/riscv64-unknown-elf/lib/rv32imac/ilp32/",
        newlib_path.display(),
    );
    println!(
        "cargo::rustc-link-search={}/riscv/riscv64-unknown-elf/lib/rv32imac/ilp32/",
        libcpp_path.display(),
    );
    println!(
        "cargo::rustc-link-search={}/rv32imac/ilp32/",
        libgcc_path.display(),
    );

    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
        .file("c_src/og_littlefs.c")
        // I know this is the grossest fix in history
        .file("../../omniglot-tock/omniglot_c_rt/sys.c")
        .include("../../third-party/littlefs/")
        .compile("liboglfs");

    println!("cargo::rustc-link-lib=lfs");
    println!("cargo::rustc-link-lib=c");
    println!("cargo::rustc-link-lib=m");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rustc-link-lib=supc++");
    println!("cargo::rustc-link-lib=gcc");

    // println!("cargo::rustc-link-lib=oglfs");
}
