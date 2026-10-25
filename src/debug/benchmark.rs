use crate::platform::timer;
use core::arch::asm;

pub fn compare<T: ?Sized, const N: usize>(
    input: &mut T,
    rounds: u32,
    mut prepare: impl FnMut(),
    tests: [fn(&mut T); N],
    mut verify: impl FnMut(&T),
) -> [u32; N] {
    assert!(rounds != 0 && N != 0);
    let mut totals = [0u64; N];

    for test in tests {
        prepare();
        test(core::hint::black_box(&mut *input));
        verify(input);
    }

    for round in 0..rounds {
        for offset in 0..N {
            let index = (round as usize + offset) % N;
            prepare();

            unsafe { asm!("", options(nostack, preserves_flags)) }
            let start = timer::micros();
            tests[index](core::hint::black_box(&mut *input));
            unsafe { asm!("", options(nostack, preserves_flags)) }
            let elapsed = timer::micros().wrapping_sub(start);

            verify(input);
            totals[index] += u64::from(elapsed);
        }
    }

    totals.map(|total| (total / u64::from(rounds)) as u32)
}
