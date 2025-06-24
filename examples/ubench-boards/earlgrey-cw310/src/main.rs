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

    #[cfg(any(
        feature = "og_eval_ubench_invoke",
        feature = "og_eval_ubench_validate",
        feature = "og_eval_ubench_upgrade",
        feature = "og_eval_ubench_callback"
    ))]
    omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
        // Try to load the ogubench Omniglot TBF binary:
        let ogubench_binary = omniglot_tock::binary::OmniglotBinary::find(
            "og_ubench",
            core::slice::from_raw_parts(
                &_sapps as *const u8,
                &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
            ),
        )
        .unwrap();

        let (rt, mut alloc, mut access) = unsafe {
            omniglot_tock::rv32i_c_rt::TockRv32iCRt::new(
                kernel::platform::chip::Chip::mpu(chip),
                ogubench_binary,
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
        let bound_rt = omniglot_tock_ubench::libubench::LibUbenchRt::new(rt).unwrap();

        // TODO: this is bad! This is creating a second instance of this
        // hardware alarm, over the same hardware peripheral. It should be
        // OK for now, as we're currently just using it to same the
        // current time, which does not incur any register writes.
        let hardware_alarm = static_init!(
            earlgrey::timer::RvTimer<ChipConfig>,
            earlgrey::timer::RvTimer::new()
        );

        // Invoke benchmarks:
        #[cfg(feature = "og_eval_ubench_invoke")]
        omniglot_tock_ubench::run_ubench_invoke(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
            &mut || kernel::platform::chip::Chip::mpu(chip).request_reconfiguration(),
        );

        // Validate benchmarks:
        #[cfg(feature = "og_eval_ubench_validate")]
        {
            omniglot_tock_ubench::run_ubench_validate_bytes(
                &bound_rt,
                &mut alloc,
                &mut access,
                hardware_alarm,
            );
            omniglot_tock_ubench::run_ubench_validate_str(
                &bound_rt,
                &mut alloc,
                &mut access,
                hardware_alarm,
            );
        }

        // Upgrade benchmark:
        #[cfg(feature = "og_eval_ubench_upgrade")]
        omniglot_tock_ubench::run_ubench_upgrade(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
        );

        // Callback benchmark:
        #[cfg(feature = "og_eval_ubench_callback")]
        omniglot_tock_ubench::run_ubench_callback(
            &bound_rt,
            &mut alloc,
            &mut access,
            hardware_alarm,
        );
    });

    // -------------------------------------------------------------------------
    // Setup time benchmark:
    #[cfg(feature = "og_eval_ubench_setup")]
    {
        use kernel::hil::time::Time;

        // TODO: this is bad! This is creating a second instance of this
        // hardware alarm, over the same hardware peripheral. It should be
        // OK for now, as we're currently just using it to same the
        // current time, which does not incur any register writes.
        let hardware_alarm = static_init!(
            earlgrey::timer::RvTimer<ChipConfig>,
            earlgrey::timer::RvTimer::new()
        );

        const SETUP_ITERS: usize = 10_000;

        let start_unsafe = hardware_alarm.now();
        for _ in 0..SETUP_ITERS {
            omniglot_tock_ubench::bench_args_unsafe::<0>();
        }
        let end_unsafe = hardware_alarm.now();

        let start_og = hardware_alarm.now();
        for _ in 0..SETUP_ITERS {
            omniglot::id::lifetime::OGLifetimeBranding::new(|brand| {
                // Try to load the ogubench Omniglot TBF binary:
                let ogubench_binary = omniglot_tock::binary::OmniglotBinary::find(
                    "og_ubench",
                    core::slice::from_raw_parts(
                        &_sapps as *const u8,
                        &_eapps as *const u8 as usize - &_sapps as *const u8 as usize,
                    ),
                )
                .unwrap();

                let (rt, mut alloc, mut access) = unsafe {
                    omniglot_tock::rv32i_c_rt::TockRv32iCRt::new(
                        kernel::platform::chip::Chip::mpu(chip),
                        ogubench_binary,
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
                let bound_rt = omniglot_tock_ubench::libubench::LibUbenchRt::new(rt).unwrap();

                // Run a single function:
                omniglot_tock_ubench::bench_args_og::<0, _, _, _>(
                    &bound_rt,
                    &mut alloc,
                    &mut access,
                );
            });
        }
        let end_og = hardware_alarm.now();

        omniglot_tock::print_ogbench_result(
            "setup_unsafe",
            None::<()>,
            (SETUP_ITERS, start_unsafe, end_unsafe),
            hardware_alarm,
        );
        omniglot_tock::print_ogbench_result(
            "setup_og",
            None::<()>,
            (SETUP_ITERS, start_og, end_og),
            hardware_alarm,
        );
    }
    // -------------------------------------------------------------------------

    kernel::debug!("-ogbenchdone-");
    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(earlgrey, chip, None::<&kernel::ipc::IPC<0>>, &main_loop_cap);
}
