#![no_std]

use kernel::hil::time::{ConvertTicks, Ticks, Time};

// Magic:
use omniglot::foreign_memory::og_mut_ref::OGMutRef;
use omniglot::foreign_memory::og_mut_slice::OGMutSlice;
use omniglot::id::OGID;
use omniglot::markers::{AccessScope, AllocScope};
use omniglot::rt::OGRuntime;

// Includes bindgen magic:
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
pub mod libubench {
    include!(concat!(env!("OUT_DIR"), "/libubench_bindings.rs"));
}

// python3 -c 'import secrets, os, functools; randbytes_len = 8 * 1024; randstr_len = 8 * 1024; print(f"pub const RANDBYTES: [u8; {randbytes_len}] = ["); print(functools.reduce(lambda x, y: x + (f"{y[1]},\\n" if (y[0] + 1) % 8 == 0 else f"{y[1]}, "), enumerate([f"0x{b:02x}" for b in secrets.token_bytes(randbytes_len)]), "")); print("];"); randstr = os.urandom(int(randstr_len / 2)).hex(); assert(len(randstr) == randstr_len); print("pub const RANDSTR: &str = \"{}\";".format(randstr))' > src/randbytes.rs
//
// Defines a const RANDBYTES: [u8; _], and a const RANDSTR: &str
mod constants {
    include!("./randbytes.rs");
}

use libubench::LibUbench;

pub fn print_result<T: Time>(
    label: &str,
    elements: Option<usize>,
    measurement: (usize, T::Ticks, T::Ticks),
    time: &T,
) {
    let (iters, start, end) = measurement;
    assert!(end > start);
    let ticks = end.wrapping_sub(start);
    let us = time.ticks_to_us(ticks);
    kernel::debug!(
        "[{}({:?})]: {:?} ticks ({} us) for {} iters, {} ticks / iter, {} us / iter",
        label,
        elements,
        ticks,
        us,
        iters,
        (ticks.into_u32() as f32) / iters as f32,
        (us as f32) / iters as f32
    );
}

#[inline(always)]
pub fn bench_args_og<
    const ARG_COUNT: usize,
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
>(
    lib: &L,
    alloc: &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
) {
    match ARG_COUNT {
        0 => lib.ubench_nop(alloc, access).unwrap(),
        _ => panic!("Unsupported arg count: {:?}", ARG_COUNT),
    };
}

#[inline(always)]
pub fn bench_args_unsafe<const ARG_COUNT: usize>() {
    match ARG_COUNT {
        0 => unsafe { libubench::ubench_nop() },
        _ => panic!("Unsupported arg count: {:?}", ARG_COUNT),
    };
}

#[inline(never)]
pub fn bench_invoke_og<ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>, T: Time>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
    iters: usize,
    pmp_warm: bool,
    mut pmp_request_reconfiguration: impl FnMut(),
) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    // One warmup iteration, to set up the PMP:
    bench_args_og::<0, _, _, _>(lib, alloc, access);

    for _ in 0..iters {
        // If !pmp_warm, make it reconfigure for every invoke:
        if !pmp_warm {
            pmp_request_reconfiguration();
        }
        core::hint::black_box(bench_args_og::<0, _, _, _>(lib, alloc, access));
    }

    let end = time.now();

    (iters, start, end)
}

#[inline(never)]
pub fn bench_invoke_unsafe<T: Time>(time: &T, iters: usize) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    for _ in 0..iters {
        core::hint::black_box(bench_args_unsafe::<0>());
    }

    let end = time.now();

    (iters, start, end)
}

#[inline(never)]
pub fn run_ubench_invoke<
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
    T: Time,
>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
    mut pmp_request_reconfiguration: impl FnMut(),
) {
    const INVOKE_ITERS: usize = 100_000;
    let invoke_unsafe = bench_invoke_unsafe(time, INVOKE_ITERS);
    let invoke_og_cold = bench_invoke_og(
        lib,
        alloc,
        access,
        time,
        INVOKE_ITERS,
        false,
        &mut pmp_request_reconfiguration,
    );
    let invoke_og_warm = bench_invoke_og(
        lib,
        alloc,
        access,
        time,
        INVOKE_ITERS,
        true,
        &mut pmp_request_reconfiguration,
    );
    omniglot_tock::print_ogbench_result("invoke_unsafe", None::<()>, invoke_unsafe, time);
    omniglot_tock::print_ogbench_result("invoke_og_cold", None::<()>, invoke_og_cold, time);
    omniglot_tock::print_ogbench_result("invoke_og_warm", None::<()>, invoke_og_warm, time);
}

#[inline(never)]
pub fn bench_validate_bytes<
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
    T: Time,
>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    slice: OGMutSlice<'_, ID, u8>,
    time: &T,
    iters: usize,
) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    for _ in 0..iters {
        core::hint::black_box(core::hint::black_box(&slice).validate(access).unwrap());
    }

    let end = time.now();

    (iters, start, end)
}

#[inline(never)]
pub fn bench_validate_str<
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
    T: Time,
>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    slice: OGMutSlice<'_, ID, u8>,
    time: &T,
    iters: usize,
) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    for _ in 0..iters {
        core::hint::black_box(
            core::hint::black_box(&slice)
                .as_immut()
                .validate_as_str(access)
                .unwrap(),
        );
    }

    let end = time.now();

    (iters, start, end)
}

const VALIDATE_ITERS: usize = 10_000;

#[inline(never)]
pub fn run_ubench_validate_bytes<
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
    T: Time,
>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
) {
    let mut benchmarks: [(usize, Option<(usize, T::Ticks, T::Ticks)>); 3] =
        [(1, None), (64, None), (8 * 1024, None)];

    for (size, res) in benchmarks.iter_mut() {
        lib.rt()
            .allocate_stacked_slice_mut::<u8, _, _>(*size, alloc, |slice_alloc, alloc| {
                slice_alloc.copy_from_slice(&constants::RANDBYTES[..*size], access);
                *res = Some(bench_validate_bytes(
                    lib,
                    alloc,
                    access,
                    slice_alloc,
                    time,
                    VALIDATE_ITERS,
                ));
            })
            .unwrap();
    }

    for (size, res) in benchmarks.into_iter() {
        omniglot_tock::print_ogbench_result("validate_bytes", Some(size), res.unwrap(), time);
    }
}

#[inline(never)]
pub fn run_ubench_validate_str<
    ID: OGID,
    RT: OGRuntime<ID = ID>,
    L: LibUbench<ID, RT, RT = RT>,
    T: Time,
>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
) {
    let mut benchmarks: [(usize, Option<(usize, T::Ticks, T::Ticks)>); 3] =
        [(1, None), (64, None), (8 * 1024, None)];

    for (size, res) in benchmarks.iter_mut() {
        lib.rt()
            .allocate_stacked_slice_mut::<u8, _, _>(*size, alloc, |slice_alloc, alloc| {
                slice_alloc.copy_from_slice(&constants::RANDSTR.as_bytes()[..*size], access);
                *res = Some(bench_validate_str(
                    lib,
                    alloc,
                    access,
                    slice_alloc,
                    time,
                    VALIDATE_ITERS,
                ));
            })
            .unwrap();
    }

    for (size, res) in benchmarks.into_iter() {
        omniglot_tock::print_ogbench_result("validate_str", Some(size), res.unwrap(), time);
    }
}

#[inline(never)]
pub fn bench_upgrade<ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>, T: Time>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    base_allocation: *mut u8,
    time: &T,
    iters: usize,
) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    for _ in 0..iters {
        core::hint::black_box(
            OGMutRef::upgrade_from_ptr(core::hint::black_box(base_allocation), alloc).unwrap(),
        );
    }

    let end = time.now();

    (iters, start, end)
}

fn with_alloc<R, ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>>(
    lib: &L,
    alloc: &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    alloc_size: usize,
    f: impl FnOnce(&L, &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>, &mut AccessScope<RT::ID>) -> R,
) -> R {
    lib.rt()
        .allocate_stacked_slice_mut::<u8, _, _>(alloc_size, alloc, |_, alloc| f(lib, alloc, access))
        .unwrap()
}

#[rustfmt::skip]
#[inline(never)]
pub fn run_ubench_upgrade<ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>, T: Time>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
) {
    const UPGRADE_ITERS: usize = 1000;
    const ALLOC_SIZE: usize = 4;

    let mut bench_upgrade_res_1 = None;
    let mut bench_upgrade_res_8 = None;
    let mut bench_upgrade_res_64 = None;

    lib.rt().allocate_stacked_slice_mut::<u8, _, _>(ALLOC_SIZE, alloc, |base_allocation, alloc| {
        // 1 allocation!
        bench_upgrade_res_1 = Some(bench_upgrade(lib, alloc, access, base_allocation.as_ptr(), time, UPGRADE_ITERS));

    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
        // 8 allocations!
        bench_upgrade_res_8 = Some(bench_upgrade(lib, alloc, access, base_allocation.as_ptr(), time, UPGRADE_ITERS));

    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
    with_alloc(lib, alloc, access, ALLOC_SIZE, |lib, alloc, access| {
        // 64 allocations!
        bench_upgrade_res_64 = Some(bench_upgrade(lib, alloc, access, base_allocation.as_ptr(), time, UPGRADE_ITERS));
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })

    })
    })
    })
    })
    })
    })
    })
    }).unwrap();

    omniglot_tock::print_ogbench_result("upgrade", Some(1), bench_upgrade_res_1.unwrap(), time);
    omniglot_tock::print_ogbench_result("upgrade", Some(8), bench_upgrade_res_8.unwrap(), time);
    omniglot_tock::print_ogbench_result("upgrade", Some(64), bench_upgrade_res_64.unwrap(), time);
}

#[inline(never)]
pub fn bench_callback<ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>, T: Time>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    base_callback: *const RT::CallbackTrampolineFn,
    time: &T,
    iters: usize,
) -> (usize, T::Ticks, T::Ticks) {
    let start = time.now();

    for _ in 0..iters {
        core::hint::black_box(lib.ubench_invoke_callback(
            core::hint::black_box(unsafe {
                core::mem::transmute::<_, Option<unsafe extern "C" fn()>>(base_callback)
            }),
            alloc,
            access,
        ));
    }

    let end = time.now();

    (iters, start, end)
}

fn with_callback<R, ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>>(
    lib: &L,
    alloc: &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    f: impl FnOnce(&L, &mut AllocScope<'_, RT::AllocTracker<'_>, RT::ID>, &mut AccessScope<RT::ID>) -> R,
) -> R {
    lib.rt()
        .setup_callback(&mut |_, _, _, _| (), alloc, |_, alloc| {
            f(lib, alloc, access)
        })
        .unwrap()
}

#[rustfmt::skip]
#[inline(never)]
pub fn run_ubench_callback<ID: OGID, RT: OGRuntime<ID = ID>, L: LibUbench<ID, RT, RT = RT>, T: Time>(
    lib: &L,
    alloc: &mut AllocScope<RT::AllocTracker<'_>, RT::ID>,
    access: &mut AccessScope<RT::ID>,
    time: &T,
) {
    const CALLBACK_ITERS: usize = 10_000;

    let mut bench_callback_res_1 = None;
    let mut bench_callback_res_8 = None;
    let mut bench_callback_res_64 = None;

    lib.rt().setup_callback(&mut |_, _, _, _|(), alloc, |base_callback, alloc| {
        // 1 callback!
        bench_callback_res_1 = Some(bench_callback(lib, alloc, access, base_callback, time, CALLBACK_ITERS));

    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
        // 8 callbacks!
        bench_callback_res_8 = Some(bench_callback(lib, alloc, access, base_callback, time, CALLBACK_ITERS));

    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
    with_callback(lib, alloc, access, |lib, alloc, access| {
        // 64 callbacks!
        bench_callback_res_64 = Some(bench_callback(lib, alloc, access, base_callback, time, CALLBACK_ITERS));
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })
    })

    })
    })
    })
    })
    })
    })
    })
    }).unwrap();

    omniglot_tock::print_ogbench_result("callback", Some(1), bench_callback_res_1.unwrap(), time);
    omniglot_tock::print_ogbench_result("callback", Some(8), bench_callback_res_8.unwrap(), time);
    omniglot_tock::print_ogbench_result("callback", Some(64), bench_callback_res_64.unwrap(), time);
}
