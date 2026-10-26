// Replaces compiler_builtins' memcpy and memset, which move one word per
// iteration, with Rockbox's burst versions: ~1.5x faster for large blocks, up to
// 2.3x for small ones and 1.8x when misaligned (iPod classic 5.5). The builtins are weak symbols,
// so these take their place at link time. LLVM calls the ARM EABI variants
// (__aeabi_memcpy4, __aeabi_memclr4, ...), so the asm defines those too.
use core::arch::global_asm;

global_asm!(include_str!("memcpy.s"), options(raw));
global_asm!(include_str!("memset.s"), options(raw));
