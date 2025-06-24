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

pub struct OTCryptoLibHMAC<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
{
    lib: &'l L,
    alloc_scope: TakeCell<'l, AllocScope<'l, RT::AllocTracker<'l>, RT::ID>>,
    access_scope: TakeCell<'l, AccessScope<RT::ID>>,
    hmac_context: RefCell<EFCopy<otcrypto_mac_og_bindings::hmac_context_t>>,
    data_slice: OptionalCell<SubSlice<'static, u8>>,
    data_slice_mut: OptionalCell<SubSliceMut<'static, u8>>,
    digest_buf: TakeCell<'static, [u8; SHA_256_OUTPUT_LEN_BYTES]>,
    deferred_call: DeferredCall,
    client: OptionalCell<&'l dyn digest::Client<SHA_256_OUTPUT_LEN_BYTES>>,
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
    OTCryptoLibHMAC<'l, ID, RT, L>
{
    pub fn new(
        lib: &'l L,
        alloc_scope: &'l mut AllocScope<'l, RT::AllocTracker<'l>, RT::ID>,
        access_scope: &'l mut AccessScope<RT::ID>,
    ) -> Self {
        OTCryptoLibHMAC {
            lib,
            alloc_scope: TakeCell::new(alloc_scope),
            access_scope: TakeCell::new(access_scope),
            hmac_context: RefCell::new(EFCopy::zeroed()),
            data_slice: OptionalCell::empty(),
            data_slice_mut: OptionalCell::empty(),
            digest_buf: TakeCell::empty(),
            deferred_call: DeferredCall::new(),
            client: OptionalCell::empty(),
        }
    }

    fn with_hmac_context<R, F>(
        &self,
        alloc: &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>,
        access: &mut AccessScope<RT::ID>,
        f: F,
    ) -> R
    where
        F: FnOnce(
            &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>,
            &mut AccessScope<RT::ID>,
            EFMutRef<'_, ID, otcrypto_mac_og_bindings::hmac_context_t>,
        ) -> R,
    {
        let mut stored_hmac_context = self.hmac_context.borrow_mut();
        //debug!("Stored ctx pre: {:?}", &stored_hmac_context.validate_ref().unwrap());
        let res = self
            .lib
            .rt()
            .allocate_stacked_t_mut::<otcrypto_mac_og_bindings::hmac_context_t, _, _>(
                alloc,
                |stacked_context, alloc| {
                    // Copy our copy of the context into the stacked context:
                    stacked_context.write_copy(&*stored_hmac_context, access);
                    let res = f(alloc, access, stacked_context);
                    //debug!("Stacked ctx updated: {:p} {:?}", <_ as Into<*const _>>::into(stacked_context.as_ptr()), &*stacked_context.validate(access).unwrap());
                    stored_hmac_context.update_from_mut_ref(stacked_context, access);
                    res
                },
            )
            .unwrap();
        //debug!("Stored ctx post: {:?}", &stored_hmac_context.validate_ref().unwrap());
        res
    }

    fn add_data_int(&self, data: &[u8]) -> Result<(), ErrorCode> {
        let access = self.access_scope.take().unwrap();
        let alloc = self.alloc_scope.take().unwrap();

        let res = self.with_hmac_context(alloc, access, |alloc, access, hmac_context| {
            let msg_buf = otcrypto_mac_og_bindings::crypto_const_byte_buf_t {
                data: data as *const _ as *const _,
                len: data.len(),
            };
            //panic!("Adding msg buf: {}, {}, {:x?}, {:?}", data.len(), data_slice.len(), &msg_buf, &*data_slice.validate(access).unwrap());

            self.lib
                .otcrypto_hmac_update(hmac_context.as_ptr().into(), msg_buf, access)
                .unwrap();
        });

        self.access_scope.replace(access);
        self.alloc_scope.replace(alloc);

        // todo: is there a mapping or helper func for EFError -> ErrorCode?
        //res.map_err(|_| ErrorCode::FAIL)
        Ok(())
    }
}

use kernel::deferred_call::DeferredCallClient;

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>> DeferredCallClient
    for OTCryptoLibHMAC<'l, ID, RT, L>
{
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
impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
    digest::Digest<'l, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_client(&'l self, client: &'l dyn digest::Client<32>) {
        self.client.replace(client);
    }
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
    digest::DigestData<'l, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_data_client(&'l self, client: &'l dyn digest::ClientData<32>) {
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

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
    digest::DigestHash<'l, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_hash_client(&'l self, client: &'l dyn digest::ClientHash<32>) {
        // see comment for dataclient
        unimplemented!()
    }
    fn run(
        &'l self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        let alloc = self.alloc_scope.take().unwrap();
        let access = self.access_scope.take().unwrap();

        self.with_hmac_context(alloc, access, |alloc, access, hmac_context| {
            self.lib.rt().allocate_stacked_t_mut::<[u32; 256 / 32], _, _>(alloc, |tag_array, alloc| {
                self.lib.rt().allocate_stacked_t_mut::<otcrypto_mac_og_bindings::crypto_word32_buf_t, _, _>(alloc, |tag_buf, _alloc| {
                    tag_buf.write(otcrypto_mac_og_bindings::crypto_word32_buf_t {
                        data: tag_array.as_ptr().cast::<u32>().into(),
                        len: 256 / 32,
                    }, access);

                    self.lib.otcrypto_hmac_final(
                        hmac_context.as_ptr().into(),
                        tag_buf.as_ptr().into(),
                        access,
                    ).unwrap();

                    // Should be infallible, as it is an array over a primitive type:
                    let tag_array_val = tag_array.validate(access).unwrap();

                    // Copy the validated array's contents into the digest buffer,
                    // converting the u32s to u8s in the process:
                    //panic!("Hash done tag_array_val: {:x?}", &*tag_array_val);
                    tag_array_val
                        .iter()
                        .flat_map(|w| u32::to_be_bytes(*w))
                        .zip(digest.iter_mut())
                        .for_each(|(src, dst)| *dst = src);
                }).unwrap()
            }).unwrap();
        });

        // Store the digest slice and request a deferred call:
        self.digest_buf.replace(digest);
        self.deferred_call.set();

        // Return alloc and access scopes:
        self.alloc_scope.replace(alloc);
        self.access_scope.replace(access);

        Ok(())
    }
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>>
    digest::DigestVerify<'l, { SHA_256_OUTPUT_LEN_BYTES }> for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_verify_client(
        &'l self,
        client: &'l dyn digest::ClientVerify<{ SHA_256_OUTPUT_LEN_BYTES }>,
    ) {
        // see comment for dataclient
        unimplemented!()
    }
    fn verify(
        &'l self,
        compare: &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES],
    ) -> Result<(), (ErrorCode, &'static mut [u8; SHA_256_OUTPUT_LEN_BYTES])> {
        //self.run(compare)
        unimplemented!();
    }
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>> digest::HmacSha256
    for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        assert!(key.len() == 32);

        let access = self.access_scope.take().unwrap();
        let alloc = self.alloc_scope.take().unwrap();

        //self.lib.rt().allocate_stacked_t::<otcrypto_mac_og_bindings::hmac_context_t, _, _>(alloc, |hmac_context, alloc| {
        self.with_hmac_context(alloc, access, |alloc, access, hmac_context| {
            // Create a key and initialize the context with that key:
            self.lib.rt().allocate_stacked_t_mut::<otcrypto_mac_og_bindings::crypto_blinded_key_t, _, _>(alloc, |blinded_key, alloc| {
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
                    security_level: otcrypto_mac_og_bindings::crypto_key_security_level_kSecurityLevelLow,
                };

                //blinded_key_config.write(key_config_rust, &mut access);

                // Create keyblob from key and mask:
                let keyblob_words = self.lib.keyblob_num_words(key_config_rust, access)
                    .unwrap().validate().unwrap();

                self.lib.rt().allocate_stacked_slice_mut::<u32, _, _>(keyblob_words, alloc, |keyblob, alloc| {
                    self.lib.rt().allocate_stacked_t_mut::<[u32; 17], _, _>(alloc, |test_mask, alloc| {
                        test_mask.write([
                                 0x8cb847c3, 0xc6d34f36, 0x72edbf7b, 0x9bc0317f, 0x8f003c7f, 0x1d7ba049,
                                 0xfd463b63, 0xbb720c44, 0x784c215e, 0xeb101d65, 0x35beb911, 0xab481345,
                                 0xa7ebc3e3, 0x04b2a1b9, 0x764a9630, 0x78b8f9c5, 0x3f2a1d8e,
                        ], access);

                        self.lib.rt().allocate_stacked_t_mut::<[u32; 32], _, _>(alloc, |test_key, _alloc| {
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
                                .zip(test_key.iter())
                                .for_each(|(src, dst)| { dst.write(src, access); });

                            self.lib.keyblob_from_key_and_mask(
                                test_key.as_ptr().cast::<u32>().into(),
                                test_mask.as_ptr().cast::<u32>().into(),
                                key_config_rust,
                                keyblob.as_ptr().into(),
                                access,
                            ).unwrap();
                        }).unwrap();
                    }).unwrap();

                    //debug!("EF -- Produced keyblob: {:x?}", &*keyblob.validate(access).unwrap());

                    blinded_key.write(otcrypto_mac_og_bindings::crypto_blinded_key_t {
                        config: key_config_rust,
                        keyblob: keyblob.as_ptr().into(),
                        keyblob_length: keyblob_words * core::mem::size_of::<u32>(),
                        checksum: 0,
                    }, access);

                    let checksum = self.lib.integrity_blinded_checksum(blinded_key.as_ptr().into(), access)
                        .unwrap().validate().unwrap();

                    // TODO: this should really only update the inner reference! 
                    blinded_key.write(otcrypto_mac_og_bindings::crypto_blinded_key_t {
                        config: key_config_rust,
                        keyblob: keyblob.as_ptr().into(),
                        keyblob_length: keyblob_words * core::mem::size_of::<u32>(),
                        checksum: checksum,
                    }, access);
                    //debug!("Blinded checksummed key: {:?}", &*blinded_key.validate(access).unwrap());


                    // For now, I'm going to have this method init hmac too. 
                    // We may want this elsewhere
                    let res = self.lib.otcrypto_hmac_init(
                        hmac_context.as_ptr().into(),
                        blinded_key.as_ptr().into(),
                        access,
                    ).unwrap();
                    //panic!("HMAC init res: {:?}", res.validate().unwrap());

                    // todo: punting on error handling for now...
                }).unwrap();
            }).unwrap();
        });

        self.access_scope.replace(access);
        self.alloc_scope.replace(alloc);
        Ok(())
    }
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>> digest::HmacSha384
    for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}

impl<'l, ID: OGID, RT: OGRuntime<ID = ID>, L: LibOTCryptoMAC<ID, RT, RT = RT>> digest::HmacSha512
    for OTCryptoLibHMAC<'l, ID, RT, L>
{
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        unimplemented!()
    }
}
