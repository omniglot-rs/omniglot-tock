/* ogrt_header is defined by the service linker script (omniglot_layout.ld).
 * It has the following layout:
 *
 *     Field                       | Offset
 *     ------------------------------------
 *     Start of .data in flash     |      0
 *     Size of .data               |      4
 *     Start of .data in RAM       |      8
 *     Size of .bss                |     12
 *     Start of .bss in RAM        |     16
 */

/* omniglot_init is executed by the kernel to have it initialize its memory.
 * The kernel passes the following arguments:
 *
 *     a0  Pointer to the ogrt_header.
 *
 * After initialization, we are expected to return to the kernel by invoking
 * `ret` (`jalr x0, x1, 0`), where `a0` is used to indicate the initialization
 * status. `a0` = 0 is interpreted as successful initialization, all other
 * values indicate an error. `a1` must indicate the new stack pointer top, to
 * be used by the Omniglot runtime for stacking values, or
 * NULL if the top of the assigned memory region shall be used.
 */

.section .omniglot_init
.globl omniglot_init
omniglot_init:
        /* Make sure all of the provided parameters are word-aligned, but only
	 * if there is actually any data to copy (otherwise the linker will
	 * place them whereever): */
        lw   t1, 1*4(a0)             /* remaining = ogrt_header.data_size */
	beqz t1, .Lzero_bss          /* short circuit if we don't have data to copy */
        /* andi t3, t1, 3 */         /* TODO: for some reason, the PhysAddr of load segments can be aligned to less than 4 bytes? */
        /* bnez t3, .Linit_error */

        lw   t0, 0*4(a0)             /* src = ogrt_header.data_flash */
        /* andi t3, t0, 3 */
        /* bnez t3, .Linit_error */

        lw   t2, 2*4(a0)             /* dest = ogrt_header.data_ram */
        /* andi t3, t2, 3 */
        /* bnez t3, .Linit_error */

        /* Copy data */
        beqz t1, .Lzero_bss          /* Jump to zero_bss if remaining is zero */

.Ldata_loop_body:
        lb   t3, 0(t0)               /* t3 = *src */
        sb   t3, 0(t2)               /* *dest = t3 */
        addi t0, t0, 1               /* src += 1 */
        addi t1, t1, -1              /* remaining -= 1 */
        addi t2, t2, 1               /* dest += 1 */
        bnez t1, .Ldata_loop_body    /* Loop if there's still data remaining */

.Lzero_bss:
        lw   t0, 3*4(a0)             /* remaining = rt_ogrt_header.bss_size */
        lw   t1, 4*4(a0)             /* dest = rt_ogrt_header.bss_start */
        add  t2, t1, t0              /* end = dest + remaining */

        /* Zero BSS */
        beq  t1, t2, .Linit_done     /* Jump to init_done if no data to copy */

.Lbss_loop_body:
        sb   zero, 0(t1)             /* *dest = zero */
        addi t1, t1, 1               /* dest += 1 */
        bne  t1, t2, .Lbss_loop_body /* Iterate again if dest != end */

.Linit_done:
        la   sp, _stack_top          /* Tell the runtime the location of _stack_top */
        li   a0, 0                   /* Report no error */
        ret

/* .Linit_error: */
/*         li   a0, 1 */
/*         ret */
