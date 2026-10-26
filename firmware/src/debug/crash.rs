use core::fmt::Write;
use core::panic::PanicInfo;

use crate::drivers::lcd;
use crate::graphics::color::{BLACK, RED, WHITE};
use crate::graphics::framebuffer;
use crate::graphics::text::Text;
use crate::platform::stack;

// same order as exception.s: kind indexes KINDS, the frame follows REGISTERS
const KINDS: [&str; 6] = ["undefined instruction", "swi", "prefetch abort", "data abort", "irq", "fiq"];
const REGISTERS: [&str; 17] = [
    "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "sp", "lr", "pc", "psr",
];

static mut CRASHED: bool = false;

#[unsafe(no_mangle)]
extern "C" fn exception(kind: usize, frame: &[u32; 17]) -> ! {
    report(KINDS.get(kind).copied().unwrap_or("unknown exception"), |text| {
        for (index, (name, value)) in REGISTERS.iter().zip(frame).enumerate() {
            let end = if index % 2 == 1 { "\n" } else { "  " };
            let _ = write!(text, "{name:<4}{value:08X}{end}");
        }
    })
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    report("panic", |text| {
        if let Some(location) = info.location() {
            let _ = writeln!(text, "{}:{}\n", location.file(), location.line());
        }
        let _ = write!(text, "{}", info.message());
    })
}

fn report(title: &str, body: impl FnOnce(&mut Text)) -> ! {
    // a fault while reporting would recurse forever, so a second one just stops
    if !unsafe { CRASHED } {
        unsafe { CRASHED = true };

        // main still owns the framebuffer, but never runs again
        let fb = unsafe { framebuffer::steal() };
        fb.clear();
        let mut text = Text::new(fb);
        text.set_scale(2);
        text.move_to(4, 4);
        text.set_color(RED, BLACK);
        let _ = writeln!(text, "{title}\n");
        text.set_color(WHITE, BLACK);
        body(&mut text);

        text.newline();
        text.set_scale(1);
        text.newline();
        for stack in stack::usage() {
            // used down to its bottom: it probably overflowed
            text.set_color(if stack.used == stack.size { RED } else { WHITE }, BLACK);
            let _ = writeln!(text, "{:<10}{:>5} of {:>5} bytes", stack.name, stack.used, stack.size);
        }

        lcd::init().update(fb.pixels());
    }

    loop {
        core::hint::spin_loop();
    }
}
