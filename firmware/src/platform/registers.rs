use core::ptr::{read_volatile, write_volatile};

pub struct Register32 {
    address: usize,
}

impl Register32 {
    pub const fn new(address: usize) -> Self {
        Self { address }
    }

    pub const fn address(&self) -> usize {
        self.address
    }

    #[inline(always)]
    pub unsafe fn read(&self) -> u32 {
        unsafe { read_volatile(self.address as *const u32) }
    }

    #[inline(always)]
    pub unsafe fn write(&self, value: u32) {
        unsafe { write_volatile(self.address as *mut u32, value) }
    }

    #[inline(always)]
    pub unsafe fn modify(&self, change: impl FnOnce(u32) -> u32) {
        unsafe { self.write(change(self.read())) }
    }
}

pub struct Register16 {
    address: usize,
}

impl Register16 {
    pub const fn new(address: usize) -> Self {
        Self { address }
    }

    #[inline(always)]
    pub unsafe fn read(&self) -> u16 {
        unsafe { read_volatile(self.address as *const u16) }
    }
}

pub struct Register8 {
    address: usize,
}

impl Register8 {
    pub const fn new(address: usize) -> Self {
        Self { address }
    }

    #[inline(always)]
    pub unsafe fn read(&self) -> u8 {
        unsafe { read_volatile(self.address as *const u8) }
    }

    #[inline(always)]
    pub unsafe fn write(&self, value: u8) {
        unsafe { write_volatile(self.address as *mut u8, value) }
    }
}

pub const DEV_INIT1: Register32 = Register32::new(0x7000_0010);
pub const DEV_RS: Register32 = Register32::new(0x6000_6004);
pub const DEV_EN: Register32 = Register32::new(0x6000_600C);
pub const PWM0_CTRL: Register32 = Register32::new(0x7000_A000);

// read-only free-running microsecond counter
pub const USEC_TIMER: Register32 = Register32::new(0x6000_5010);

pub const GPIOL_OUTPUT_VAL: Register32 = Register32::new(0x6000_D12C);

pub const GPIOA_ENABLE: Register32 = Register32::new(0x6000_D000);
pub const GPIOA_OUTPUT_EN: Register32 = Register32::new(0x6000_D010);
pub const GPIOA_INPUT_VAL: Register32 = Register32::new(0x6000_D030);

pub const GPIOC_ENABLE: Register32 = Register32::new(0x6000_D008);
pub const GPIOC_OUTPUT_EN: Register32 = Register32::new(0x6000_D018);
pub const GPIOG_ENABLE: Register32 = Register32::new(0x6000_D088);
pub const GPIOH_ENABLE: Register32 = Register32::new(0x6000_D08C);
pub const GPIOI_ENABLE: Register32 = Register32::new(0x6000_D100);
pub const GPIOK_ENABLE: Register32 = Register32::new(0x6000_D108);
pub const GPO32_VAL: Register32 = Register32::new(0x7000_0080);
pub const GPO32_ENABLE: Register32 = Register32::new(0x7000_0084);

pub const IDE0_PRI_TIMING0: Register32 = Register32::new(0xC300_0000);
pub const IDE0_PRI_TIMING1: Register32 = Register32::new(0xC300_0004);
pub const IDE0_CFG: Register32 = Register32::new(0xC300_0028);
// ATA registers of the IDE controller, one per word
pub const ATA_DATA: Register16 = Register16::new(0xC300_01E0);
pub const ATA_ERROR: Register8 = Register8::new(0xC300_01E4);
pub const ATA_NSECTOR: Register8 = Register8::new(0xC300_01E8);
pub const ATA_LBA_LOW: Register8 = Register8::new(0xC300_01EC);
pub const ATA_LBA_MID: Register8 = Register8::new(0xC300_01F0);
pub const ATA_LBA_HIGH: Register8 = Register8::new(0xC300_01F4);
pub const ATA_SELECT: Register8 = Register8::new(0xC300_01F8);
// the command when written, the status when read
pub const ATA_COMMAND: Register8 = Register8::new(0xC300_01FC);
// the control when written; reading gives the status without acknowledging an interrupt
pub const ATA_CONTROL: Register8 = Register8::new(0xC300_03F8);

pub const BCM_DATA32: Register32 = Register32::new(0x3000_0000);
pub const BCM_WR_ADDR32: Register32 = Register32::new(0x3001_0000);
pub const BCM_RD_ADDR32: Register32 = Register32::new(0x3002_0000);
// BCM control accesses are 16-bit.
pub const BCM_CONTROL: *mut u16 = 0x3003_0000 as *mut u16;

pub const CLOCK_SOURCE: Register32 = Register32::new(0x6000_6020);
pub const PLL_CONTROL: Register32 = Register32::new(0x6000_6034);
pub const PLL_STATUS: Register32 = Register32::new(0x6000_603C);

pub const CPU_INT_STAT: Register32 = Register32::new(0x6000_4000);
pub const INT_FORCED_CLR: Register32 = Register32::new(0x6000_401C);
pub const CPU_INT_EN: Register32 = Register32::new(0x6000_4024);
pub const CPU_INT_DIS: Register32 = Register32::new(0x6000_4028);
pub const COP_INT_DIS: Register32 = Register32::new(0x6000_4038);
pub const CPU_HI_INT_STAT: Register32 = Register32::new(0x6000_4100);
pub const HI_INT_FORCED_CLR: Register32 = Register32::new(0x6000_411C);
pub const CPU_HI_INT_EN: Register32 = Register32::new(0x6000_4124);
pub const CPU_HI_INT_DIS: Register32 = Register32::new(0x6000_4128);
pub const COP_HI_INT_DIS: Register32 = Register32::new(0x6000_4138);

// opto controller that reads the clickwheel
pub const CLICKWHEEL_CTRL: Register32 = Register32::new(0x7000_C100);
pub const CLICKWHEEL_STATUS: Register32 = Register32::new(0x7000_C104);
pub const CLICKWHEEL_DATA: Register32 = Register32::new(0x7000_C140);

pub const CACHE_CTL: Register32 = Register32::new(0x6000_C000);
pub const CACHE_PRIORITY: Register32 = Register32::new(0x6000_6044);
pub const CACHE_MASK: Register32 = Register32::new(0xF000_F040);
pub const CACHE_OPERATION: Register32 = Register32::new(0xF000_F044);

pub const CPU_CTRL: Register32 = Register32::new(0x6000_7000);
pub const DEV_INIT2: Register32 = Register32::new(0x7000_0020);
pub const DEV_TIMING1: Register32 = Register32::new(0x7000_0034);
