#![no_std]

// Magic:
// use omniglot::branding::OGID;
// use omniglot::rt::OGRuntime;
// use omniglot::types::{AccessScope, AllocScope};

// Includes bindgen magic:
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
pub mod libotcrypto_bindings {
    include!(concat!(env!("OUT_DIR"), "/libotcrypto_bindings.rs"));
}

pub mod og_otcrypto_hmac;

// pub mod og_otcrypto_hmac_nocopy;
pub mod unsafe_otcrypto_hmac;
// pub mod unsafe_otcrypto_hmac_validate;

pub mod hmac_bench;
