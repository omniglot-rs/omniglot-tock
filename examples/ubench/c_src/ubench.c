#include "ubench.h"

void ubench_nop(void) {}

void ubench_invoke_callback(void (*callback_fn)()) {
    callback_fn();
}
