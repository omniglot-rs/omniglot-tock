#include "omniglot_otcrypto_tbf.h"

otcrypto_hmac_context_t global_hmac_context;
otcrypto_hmac_context_t* get_global_hmac_context_ptr() {
    return &global_hmac_context;
}

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".omniglot_hdr")))
omniglot_fntab[8] = {
  /* 0 */ (fnptr) keyblob_num_words,
  /* 1 */ (fnptr) keyblob_from_key_and_mask,
  /* 2 */ (fnptr) integrity_blinded_checksum,
  /* 3 */ (fnptr) otcrypto_hmac_init,
  /* 4 */ (fnptr) otcrypto_hmac_update,
  /* 5 */ (fnptr) otcrypto_hmac_final,
  /* 6 */ (fnptr) entropy_complex_init,
  /* 7 */ (fnptr) get_global_hmac_context_ptr,
};

__attribute__ ((section (".omniglot_hdr")))
const size_t omniglot_fntab_length = sizeof(omniglot_fntab);
