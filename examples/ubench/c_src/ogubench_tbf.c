#include "ubench.h"
#include <stddef.h>

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".omniglot_hdr")))
omniglot_fntab[2] = {
    /* 0 */ (fnptr) ubench_nop,
    /* 1 */ (fnptr) ubench_invoke_callback,
};

__attribute__ ((section (".omniglot_hdr")))
const size_t omniglot_fntab_length = sizeof(omniglot_fntab);
