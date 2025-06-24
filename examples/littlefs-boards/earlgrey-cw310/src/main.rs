#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use earlgrey_board_lib::{ChipConfig, EarlGreyChip};
use omniglot::rt::OGRuntime;
use kernel::platform::mpu;
use kernel::{capabilities, create_capability, static_init};
use kernel::debug;

/// Main function.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup and RAM initialization.
#[no_mangle]
pub unsafe fn main() {
    extern "C" {
        static _sapps: u8;
        static _eapps: u8;

        static mut _ogram_start: u8;
        static mut _ogram_end: u32;
    }

    let (board_kernel, earlgrey, chip, _peripherals) = earlgrey_board_lib::start();

    // TODO: this is bad! This is creating a second instance of this
    // hardware alarm, over the same hardware peripheral. It should be
    // OK for now, as we're currently just using it to same the
    // current time, which does not incur any register writes.
    let hardware_alarm = static_init!(
        earlgrey::timer::RvTimer<ChipConfig>,
        earlgrey::timer::RvTimer::new()
    );

    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        // This is unsafe, as it instantiates a runtime that can be used to run
        // foreign functions without memory protection:
        let (rt, mut alloc, mut access) = unsafe {
            omniglot::rt::mock::MockRt::new(
                false, // zero_copy_immutable
                true, // all_upgrades_valid
                omniglot::rt::mock::stack_alloc::StackAllocator::<
                    omniglot::rt::mock::stack_alloc::StackFrameAllocRiscv,
                >::new(),
                brand,
            )
        };

        // Create a "bound" runtime
        let bound_rt = omniglot_tock_littlefs::littlefs_bindings::LibLittleFSRt::new(rt).unwrap();

        // Run a test:
        omniglot_tock_littlefs::test_littlefs(
            if cfg!(feature = "og_eval_disable_checks") { "og_mock_unchecked" } else { "og_mock_checked" },
            &bound_rt, &mut alloc, &mut access, hardware_alarm);
    });

    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        // Try to load the og_littlefs Omniglot TBF binary:
        let efdemo_binary = omniglot_tock::binary::OmniglotBinary::find(
            "og_littlefs",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        )
        .unwrap();

        let (rt, mut alloc, mut access) = unsafe {
            omniglot_tock::rv32i_c_rt::TockRv32iCRt::new(
                kernel::platform::chip::Chip::mpu(chip),
                efdemo_binary,
                core::ptr::addr_of_mut!(_ogram_start) as *mut (),
                core::ptr::addr_of!(_ogram_end) as usize
                    - core::ptr::addr_of!(_ogram_start) as usize,
                // Expose no addl. MPU regions:
                [].into_iter(),
                brand,
            )
        }
        .unwrap();

        // Create a "bound" runtime
        let bound_rt = omniglot_tock_littlefs::littlefs_bindings::LibLittleFSRt::new(rt).unwrap();

        // Run a test:
        omniglot_tock_littlefs::test_littlefs(
            if cfg!(feature = "og_eval_disable_checks") { "og_pmp_unchecked" } else { "og_pmp_checked" },
            &bound_rt, &mut alloc, &mut access, hardware_alarm);
    });

    // Load-bearing, otherwise the binary doesn't fit in flash
    panic!("-ogbenchdone-");

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
