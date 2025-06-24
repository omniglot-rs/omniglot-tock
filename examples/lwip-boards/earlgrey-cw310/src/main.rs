#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use earlgrey_board_lib::{ChipConfig, EarlGreyChip};
use kernel::platform::mpu;
use kernel::{capabilities, create_capability, static_init};
use omniglot::rt::OGRuntime;

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

    #[cfg(not(any(feature = "og_mock", feature = "og_pmp")))]
    compile_error!("Must select either og_mock or og_pmp feature!");

    #[cfg(feature = "og_mock")]
    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        // This is unsafe, as it instantiates a runtime that can be used to run
        // foreign functions without memory protection:
        let (rt, mut alloc, mut access) = unsafe {
            omniglot::rt::mock::MockRt::new(
                false, // zero_copy_immutable
                true,  // all_upgrades_valid
                omniglot::rt::mock::stack_alloc::StackAllocator::<
                    omniglot::rt::mock::stack_alloc::StackFrameAllocRiscv,
                >::new(),
                brand,
            )
        };

        // Create a "bound" runtime
        let bound_rt = omniglot_tock_lwip::lwip_bindings::LibLwipRt::new(rt).unwrap();

        // TODO: this is bad! This is creating a second instance of this
        // hardware alarm, over the same hardware peripheral. It should be
        // OK for now, as we're currently just using it to same the
        // current time, which does not incur any register writes.
        let hardware_alarm = static_init!(
            earlgrey::timer::RvTimer<ChipConfig>,
            earlgrey::timer::RvTimer::new()
        );

        // Callback into benchmark function:
        omniglot_tock_lwip::run(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
            omniglot_tock_lwip::lwip_bindings::netif_input as *const _,
            omniglot_tock_lwip::lwip_bindings::etharp_output as *const _,
            if cfg!(feature = "og_eval_disable_checks") {
                "og_mock_unchecked"
            } else {
                "og_mock_checked"
            },
        );
    });

    #[cfg(feature = "og_pmp")]
    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        // Try to load the og_lwip Omniglot TBF binary:
        let og_lwip_binary = omniglot_tock::binary::OmniglotBinary::find(
            "og_lwip",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        )
        .unwrap();

        let (rt, mut alloc, mut access) = unsafe {
            omniglot_tock::rv32i_c_rt::TockRv32iCRt::new(
                kernel::platform::chip::Chip::mpu(chip),
                og_lwip_binary,
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
        let bound_rt = omniglot_tock_lwip::lwip_bindings::LibLwipRt::new(rt).unwrap();

        // TODO: this is bad! This is creating a second instance of this
        // hardware alarm, over the same hardware peripheral. It should be
        // OK for now, as we're currently just using it to same the
        // current time, which does not incur any register writes.
        let hardware_alarm = static_init!(
            earlgrey::timer::RvTimer<ChipConfig>,
            earlgrey::timer::RvTimer::new()
        );

        // Callback into benchmark function:
        omniglot_tock_lwip::run(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
            // netif_input symbol
            bound_rt.lookup_symbol(3).unwrap(),
            // etharp_output symbol
            bound_rt.lookup_symbol(14).unwrap(),
            if cfg!(feature = "og_eval_disable_checks") {
                "og_pmp_unchecked"
            } else {
                "og_pmp_checked"
            },
        );
    });

    // Load-bearing, otherwise the binary doesn't fit in flash
    panic!("-ogbenchdone-"); // indicate that eval is done

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
