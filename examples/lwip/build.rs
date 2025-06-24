use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=lwip");

    let cflags = std::env::var("OG_BINDGEN_CFLAGS").expect("Please set OG_BINDGEN_CFLAGS");

    let workspace_root_dir = Path::new("../..").canonicalize().unwrap();
    let lwip_src = workspace_root_dir
        .join("third-party/lwip")
        .canonicalize()
        .expect("third-party/lwip does not exist, did you check out the submodule?");

    println!("cargo:include={}", lwip_src.display());
    let bindings = bindgen::Builder::default()
        .header(&format!("{}/include/lwip/init.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/netif.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/dhcp.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/timeouts.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/tcp.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/udp.h", lwip_src.display()))
        .header(&format!("{}/include/lwip/ip_addr.h", lwip_src.display()))
        .clang_arg(format!("-I{}/include", lwip_src.display()))
        .clang_arg("-I./config")
        // TODO: this is brittle and will break on args that have spaces in them!
        .clang_args(cflags.split(" "))
        .rustfmt_configuration_file(Some(
            PathBuf::from("./rustfmt-bindgen.toml")
                .canonicalize()
                .unwrap(),
        ))
        .omniglot_configuration_file(Some(
            PathBuf::from("./lwip.omniglot.toml")
                .canonicalize()
                .unwrap(),
        ))
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("liblwip_bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=config");
    cc::Build::new()
        .compiler("riscv32-none-elf-gcc")
        //.file(&format!("{}/core/altcp_alloc.c", lwip_src.display()))
        //.file(&format!("{}/core/altcp.c", lwip_src.display()))
        .file(&format!("{}/core/def.c", lwip_src.display()))
        .file(&format!("{}/core/inet_chksum.c", lwip_src.display()))
        .file(&format!("{}/core/init.c", lwip_src.display()))
        .file(&format!("{}/core/ip.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/acd.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/autoip.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/dhcp.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/etharp.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/icmp.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/ip4.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/ip4_addr.c", lwip_src.display()))
        .file(&format!("{}/core/ipv4/ip4_frag.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/icmp6.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/ip6.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/ip6_addr.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/ip6_frag.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/mld6.c", lwip_src.display()))
        .file(&format!("{}/core/ipv6/nd6.c", lwip_src.display()))
        .file(&format!("{}/core/mem.c", lwip_src.display()))
        .file(&format!("{}/core/memp.c", lwip_src.display()))
        .file(&format!("{}/core/netif.c", lwip_src.display()))
        .file(&format!("{}/core/pbuf.c", lwip_src.display()))
        .file(&format!("{}/core/raw.c", lwip_src.display()))
        //.file(&format!("{}/core/stats.c", lwip_src.display()))
        //.file(&format!("{}/core/sys.c", lwip_src.display())))
        .file(&format!("{}/core/tcp.c", lwip_src.display()))
        .file(&format!("{}/core/tcp_in.c", lwip_src.display()))
        .file(&format!("{}/core/tcp_out.c", lwip_src.display()))
        .file(&format!("{}/core/timeouts.c", lwip_src.display()))
        .file(&format!("{}/core/udp.c", lwip_src.display()))
        .file(&format!("{}/netif/ethernet.c", lwip_src.display()))
        .include(&format!("{}/include", lwip_src.display()))
        .include("config")
        .warnings(false)
        .compile("liblwip.a");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lwip.omniglot.toml");
}
