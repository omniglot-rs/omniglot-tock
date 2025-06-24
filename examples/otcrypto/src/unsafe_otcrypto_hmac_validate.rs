use core::cell::RefCell;

use kernel::debug;
use kernel::deferred_call::DeferredCall;
use kernel::hil::digest;
use kernel::hil::digest::DigestHash;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::SubSlice;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::ErrorCode;

use omniglot::id::OGID;
use omniglot::rt::OGRuntime;
use omniglot::markers::{AccessScope, AllocScope};
use omniglot::foreign_memory::og_copy::OGCopy;
use omniglot::foreign_memory::og_mut_ref::OGMutRef;

use crate::otcrypto_mac_og_bindings::{self, LibOTCryptoMAC};

const SHA_256_OUTPUT_LEN_BYTES: usize = 32;

pub struct OTCryptoLibHMAC<'a> {
    hmac_context: RefCell<otcrypto_mac_og_bindings::hmac_context_t>,
    data_slice: OptionalCell<SubSlice<'static, u8>>,
    data_slice_mut: OptionalCell<SubSliceMut<'static, u8>>,
    digest_buf: TakeCell<'static, [u8; SHA_256_OUTPUT_LEN_BYTES]>,
    deferred_call: DeferredCall,
    client: OptionalCell<&'a dyn digest::Client<SHA_256_OUTPUT_LEN_BYTES>>,
}

impl OTCryptoLibHMAC<'_> {
    pub fn new() -> Self {
        OTCryptoLibHMAC {
            hmac_context: RefCell::new(otcrypto_mac_og_bindings::hmac_context_t {
                inner: otcrypto_mac_og_bindings::hash_context_t {
                    mode: 0,
                    data: [0; 52],
                },
                outer: otcrypto_mac_og_bindings::hash_context_t {
                    mode: 0,
                    data: [0; 52],
                },
            }),
            data_slice: OptionalCell::empty(),
            data_slice_mut: OptionalCell::empty(),
            digest_buf: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
        }
    }

    fn with_hmac_context<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut otcrypto_mac_og_bindings::hmac_context_t) -> R,
    {
        let mut stored_hmac_context = self.hmac_context.borrow_mut();
        f(&mut *stored_hmac_context)
    }

    fn add_data_int(&self, data: &[u8]) -> Result<(), ErrorCode> {
        let res = self.with_hmac_context(|hmac_context| {
            let msg_buf = otcrypto_mac_og_bindings::crypto_const_byte_buf_t {
                data: data.as_ptr(),
                len: data.len(),
            };
            //panic!("Adding msg buf: {}, {}, {:x?}, {:?}", data.len(), data_slice.len(), &msg_buf, &*data_slice.validate(access).unwrap());

            unsafe {
                otcrypto_mac_og_bindings::otcrypto_hmac_update(hmac_context as *mut _, msg_buf)
            };
        });

        Ok(())
    }
}

use kernel::deferred_call::DeferredCallClient;

impl DeferredCallClient for OTCryptoLibHMAC<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        match (
            self.data_slice.take(),
            self.data_slice_mut.take(),
            self.digest_buf.take(),
        ) {
            (Some(data_slice), None, None) =>
            /* data slice */
            {
                self.client
                    .map(move |c| c.add_data_done(Ok(()), data_slice));
            }

            (None, Some(data_slice_mut), None) =>
            /* data slice mut */
            {
                self.client
                    .map(move |c| c.add_mut_data_done(Ok(()), data_slice_mut));
            }

            (None, None, Some(digest_buf)) =>
            /* hash done */
            {
                self.client.map(move |c| c.hash_done(Ok(()), digest_buf));
            }

            (None, None, None) => {
                unimplemented!("Unexpected deferred call!");
            }

            _ => {
                unimplemented!("Unhandled deferred call or multiple outstanding!");
            }
        }
    }
}

// HMAC Driver
impl<'a> digest::Digest<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'a> {
    fn set_client(&'a self, client: &'a dyn digest::Client<32>) {
        self.client.replace(client);
    }
}

impl<'a> digest::DigestData<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'a> {
    fn set_data_client(&'a self, client: &'a dyn digest::ClientData<32>) {
        // we do not set a client for this (this is the lowest layer)
        // mirroring hmac.rs in `chips/lowrisc/src`
        unimplemented!()
    }

    fn add_data(
        &self,
        mut data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        match self.add_data_int(data.as_slice()) {
            Err(_) => Err((ErrorCode::FAIL, data)),
            Ok(()) => {
                self.data_slice.replace(data);
                self.deferred_call.set();
                Ok(())
            }
        }
    }

    fn add_mut_data(
        &self,
        mut data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        match self.add_data_int(data.as_slice()) {
            Err(_) => Err((ErrorCode::FAIL, data)),
            Ok(()) => {
                self.data_slice_mut.replace(data);
                self.deferred_call.set();
                Ok(())
            }
        }
    }

    /// Clear the keys and any other internal state. Any pending
    /// operations terminate and issue a callback with an
    /// `ErrorCode::CANCEL`. This call does not clear buffers passed
    /// through `add_mut_data`, those are up to the client clear.
    fn clear_data(&self) {
        // it is not clear what internal state exists for encapsulated
        // functions / ot-crpyto. For now this is empty.
        unimplemented!();
    }
}

impl<'a> digest::DigestHash<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'a> {
    fn set_hash_client(&'a self, client: &'a dyn digest::ClientHash<32>) {
        // see comment for dataclient
        unimplemented!()
    }
    #[inline(never)]
    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        self.with_hmac_context(|hmac_context| {
            let mut tag_array = [0_u32; 256 / 32];
            let mut tag_buf = otcrypto_mac_og_bindings::crypto_word32_buf_t {
                data: &mut tag_array as *mut [u32; 256 / 32] as *mut u32,
                len: tag_array.len(),
            };

            unsafe {
                otcrypto_mac_og_bindings::otcrypto_hmac_final(
                    hmac_context as *mut _,
                    &mut tag_buf as *mut _,
                )
            };

            // We need to invent an AccessScope here. For this, we also need to invent a new
            // branding type. All this is super duper unsafe.
            struct UnsafeMockBranding;
            unsafe impl omniglot::id::OGID for UnsafeMockBranding {}
            let access = unsafe { omniglot::types::AccessScope::<UnsafeMockBranding>::new() };

            // Pretend that we're EF and use the validation infrastructure:
            let tag_array_og_mut_ref = unsafe {
                core::mem::transmute::<
                    &mut [u32; 256 / 32],
                    omniglot::foreign_memory::og_mut_ref::OGMutRef<UnsafeMockBranding, [u32; 256 / 32]>,
                >(&mut tag_array)
            };

            // Now, "validate" (will be a nop)
            let tag_array_og_val = tag_array_og_mut_ref.validate(&access).unwrap();

            // Copy the validated array's contents into the digest buffer,
            // converting the u32s to u8s in the process:
            //panic!("Hash done tag_array_val: {:x?}", &*tag_array_val);
            tag_array_og_val
                .iter()
                .flat_map(|w| u32::to_be_bytes(*w))
                .zip(digest.iter_mut())
                .for_each(|(src, dst)| *dst = src);
        });

        // Store the digest slice and request a deferred call:
        self.digest_buf.replace(digest);
        self.deferred_call.set();

        Ok(())
    }
}

impl<'a> digest::DigestVerify<'a, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'a> {
    fn set_verify_client(
        &'a self,
        client: &'a dyn digest::ClientVerify<{ SHA_256_OUTPUT_LEN_BYTES }>,
    ) {
        // see comment for dataclient
        unimplemented!()
    }
    fn verify(
        &'a self,
        compare: &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES])> {
        //self.run(compare)
        unimplemented!();
    }
}

impl<'a> digest::HmacSha256 for OTCryptoLibHMAC<'a> {
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        assert!(key.len() == 32);

        self.with_hmac_context(|hmac_context| {
            let key_config_rust = otcrypto_mac_og_bindings::crypto_key_config {
                version: otcrypto_mac_og_bindings::crypto_lib_version_kCryptoLibVersion1,
                key_mode: otcrypto_mac_og_bindings::key_mode_kKeyModeHmacSha256,
                key_length: 32, // HMAC-SHA256
                hw_backed: otcrypto_mac_og_bindings::hardened_bool_kHardenedBoolFalse,
                //diversification_hw_backed: otcrypto_mac_og_bindings::crypto_const_uint8_buf_t {
                //    data: core::ptr::null(),
                //    len: 0,
                //},
                exportable: otcrypto_mac_og_bindings::hardened_bool_kHardenedBoolFalse,
                security_level:
                    otcrypto_mac_og_bindings::crypto_key_security_level_kSecurityLevelLow,
            };

            //blinded_key_config.write(key_config_rust, &mut access);

            // Create keyblob from key and mask:
            let keyblob_words =
                unsafe { otcrypto_mac_og_bindings::keyblob_num_words(key_config_rust) };
            assert!(keyblob_words == 16);

            let test_mask: [u32; 17] = [
                0x8cb847c3, 0xc6d34f36, 0x72edbf7b, 0x9bc0317f, 0x8f003c7f, 0x1d7ba049, 0xfd463b63,
                0xbb720c44, 0x784c215e, 0xeb101d65, 0x35beb911, 0xab481345, 0xa7ebc3e3, 0x04b2a1b9,
                0x764a9630, 0x78b8f9c5, 0x3f2a1d8e,
            ];

            let mut test_key = [0; 32];
            key.chunks(4)
                .map(|chunk| {
                    let mut ci = chunk.iter();
                    u32::from_be_bytes([
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                        ci.next().copied().unwrap_or(0),
                    ])
                })
                .zip(test_key.iter_mut())
                .for_each(|(src, dst)| {
                    *dst = src;
                });

            let mut keyblob = [0_u32; 16];
            unsafe {
                otcrypto_mac_og_bindings::keyblob_from_key_and_mask(
                    &test_key as *const _ as *const _,
                    &test_mask as *const _ as *const _,
                    key_config_rust,
                    &mut keyblob as *mut _ as *mut _,
                )
            };

            let mut blinded_key = otcrypto_mac_og_bindings::crypto_blinded_key_t {
                config: key_config_rust,
                keyblob: &mut keyblob as *mut _ as *mut _,
                keyblob_length: keyblob_words * core::mem::size_of::<u32>(),
                checksum: 0,
            };

            let checksum = unsafe {
                otcrypto_mac_og_bindings::integrity_blinded_checksum(
                    &blinded_key as *const _ as *const _,
                )
            };

            blinded_key.checksum = checksum;
            //debug!("Blinded checksummed key: {:?}", &*blinded_key.validate(access).unwrap());

            let res = unsafe {
                otcrypto_mac_og_bindings::otcrypto_hmac_init(
                    hmac_context as *mut _,
                    &blinded_key as *const _ as *const _,
                )
            };
            //panic!("HMAC init res: {:?}", res.validate().unwrap());

            // todo: punting on error handling for now...
            //    }).unwrap();
            //}).unwrap();
        });

        Ok(())
    }
}

impl<'a> digest::HmacSha384 for OTCryptoLibHMAC<'a> {
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}

impl<'a> digest::HmacSha512 for OTCryptoLibHMAC<'a> {
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
