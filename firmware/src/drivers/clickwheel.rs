use crate::platform::irq;
use crate::platform::registers::{
    CLICKWHEEL_CTRL, CLICKWHEEL_DATA, CLICKWHEEL_STATUS, DEV_EN, DEV_INIT1, DEV_RS, GPIOA_ENABLE, GPIOA_INPUT_VAL,
    GPIOA_OUTPUT_EN,
};
use crate::platform::timer;
use crate::sync::queue::Queue;

// high source 8
pub const IRQ: u32 = 1 << 8;

const DEV_OPTO: u32 = 0x0001_0000;
const INIT_BUTTONS: u32 = 0x0004_0000;
const DATA_READY: u32 = 0x0400_0000;
// GPIO A bit 5, low while hold is on
const HOLD: u32 = 0x20;

// a packet carries the buttons and wheel when its low byte is this
const VALID: u32 = 0x8000_001A;
const VALID_MASK: u32 = 0x8000_00FF;
// rockbox ignores these without saying what they are
const IGNORED: u32 = 0x8000_003A;
const WHEEL_TOUCHED: u32 = 0x4000_0000;
const WHEEL_STEPS: i16 = 96;

// the interrupt pushes, the one Clickwheel pops
static PACKETS: Queue<u32, 32> = Queue::new();

// each value is the button's bit in a packet
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Select = 0x0100,
    Next = 0x0200,
    Previous = 0x0400,
    Play = 0x0800,
    Menu = 0x1000,
}

impl Button {
    pub const ALL: [Button; 5] = [Button::Select, Button::Next, Button::Previous, Button::Play, Button::Menu];

    fn mask(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Pressed(Button),
    Released(Button),
    Touched,
    Untouched,
    // wheel steps since the previous packet, clockwise is positive
    Scrolled(i8),
    Hold(bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct State {
    buttons: u32,
    wheel: Option<u8>,
}

impl State {
    const IDLE: State = State { buttons: 0, wheel: None };

    fn pressed(&self, button: Button) -> bool {
        self.buttons & button.mask() != 0
    }

    // position 0 to 95 while touched, clockwise increases
    fn wheel(&self) -> Option<u8> {
        self.wheel
    }
}

fn decode(packet: u32) -> Option<State> {
    if packet & VALID_MASK != VALID {
        return None;
    }
    let wheel = (packet & WHEEL_TOUCHED != 0).then(|| ((packet >> 16) & 0x7F) as u8);
    Some(State { buttons: packet & 0x1F00, wheel })
}

// the shortest way round, so going from 95 to 2 is 3 steps instead of -93
fn steps(from: u8, to: u8) -> i8 {
    let delta = i16::from(to) - i16::from(from);
    if delta > WHEEL_STEPS / 2 {
        (delta - WHEEL_STEPS) as i8
    } else if delta < -WHEEL_STEPS / 2 {
        (delta + WHEEL_STEPS) as i8
    } else {
        delta as i8
    }
}

// reads the packet queue, which allows one reader: call init only once
pub struct Clickwheel {
    last: State,
    hold: bool,
}

impl Clickwheel {
    // turns every packet since the last call into events
    pub fn poll(&mut self, mut handle: impl FnMut(Event)) {
        let hold = hold();
        if hold != self.hold {
            self.hold = hold;
            handle(Event::Hold(hold));
        }

        while let Some(packet) = unsafe { PACKETS.pop() } {
            let Some(state) = decode(packet) else { continue };
            for button in Button::ALL {
                match (self.last.pressed(button), state.pressed(button)) {
                    (false, true) => handle(Event::Pressed(button)),
                    (true, false) => handle(Event::Released(button)),
                    _ => {}
                }
            }
            match (self.last.wheel(), state.wheel()) {
                (None, Some(_)) => handle(Event::Touched),
                (Some(_), None) => handle(Event::Untouched),
                (Some(from), Some(to)) if from != to => handle(Event::Scrolled(steps(from, to))),
                _ => {}
            }
            self.last = state;
        }
    }

    // packets lost because the queue was full
    pub fn dropped(&self) -> u32 {
        PACKETS.dropped()
    }
}

pub fn init() -> Clickwheel {
    unsafe {
        // power the opto controller and reset it
        DEV_EN.modify(|value| value | DEV_OPTO);
        DEV_RS.modify(|value| value | DEV_OPTO);
        timer::delay_us(5);
        DEV_RS.modify(|value| value & !DEV_OPTO);
        DEV_INIT1.modify(|value| value | INIT_BUTTONS);

        CLICKWHEEL_CTRL.write(0xC00A_1F00);
        CLICKWHEEL_STATUS.write(0x0100_0000);

        GPIOA_ENABLE.modify(|value| value | HOLD);
        GPIOA_OUTPUT_EN.modify(|value| value & !HOLD);
    }
    irq::enable_hi_source(IRQ);
    Clickwheel { last: State::IDLE, hold: hold() }
}

// called from irq_dispatch
pub fn interrupt() {
    irq::disable_hi_source(IRQ);
    timer::delay_us(50);

    unsafe {
        if CLICKWHEEL_STATUS.read() & DATA_READY != 0 {
            let packet = CLICKWHEEL_DATA.read();
            if packet & VALID_MASK == VALID {
                // the interrupt is the only pusher
                PACKETS.push(packet);
            } else if packet & VALID_MASK != IGNORED && packet != u32::MAX {
                // an error: rockbox restarts the controller
                timer::delay_us(2000);
                CLICKWHEEL_CTRL.modify(|value| value & !0x6000_0000);
                // rockbox writes this bit here, probably an acknowledge
                CLICKWHEEL_STATUS.modify(|value| value | 0x0400_0000);
                CLICKWHEEL_CTRL.modify(|value| value | 0x6000_0000);
            }
        }

        // ready for the next packet
        CLICKWHEEL_STATUS.modify(|value| value | 0x0C00_0000);
        CLICKWHEEL_CTRL.write(0x400A_1F00);
    }
    irq::enable_hi_source(IRQ);
}

fn hold() -> bool {
    unsafe { GPIOA_INPUT_VAL.read() & HOLD == 0 }
}
