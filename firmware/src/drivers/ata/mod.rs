use core::fmt;

use fat::{BlockDevice, SectorBuffer};

use crate::platform::gpio::gpio_clear;
use crate::platform::registers::{
    ATA_COMMAND, ATA_CONTROL, ATA_DATA, ATA_ERROR, ATA_LBA_HIGH, ATA_LBA_LOW, ATA_LBA_MID, ATA_NSECTOR, ATA_SELECT,
    DEV_EN, GPIOG_ENABLE, GPIOH_ENABLE, GPIOI_ENABLE, GPIOK_ENABLE, GPO32_VAL, IDE0_CFG, IDE0_PRI_TIMING0,
    IDE0_PRI_TIMING1,
};
use crate::platform::timer;

const DEV_IDE0: u32 = 0x0200_0000;
// GPO32 bit 30, low while the drive is powered
const IDE_POWER_OFF: u32 = 0x4000_0000;

const STATUS_BSY: u8 = 0x80;
const STATUS_RDY: u8 = 0x40;
const STATUS_DF: u8 = 0x20;
const STATUS_DRQ: u8 = 0x08;
const STATUS_ERR: u8 = 0x01;
const CONTROL_NIEN: u8 = 0x02;
const CONTROL_SRST: u8 = 0x04;
const SELECT_LBA: u8 = 0x40;
const CMD_READ_SECTORS: u8 = 0x20;
const CMD_IDENTIFY: u8 = 0xEC;
// the most sectors one READ SECTORS command can transfer
const MAX_SECTORS: usize = 256;

#[derive(Clone, Copy, Debug)]
pub enum Wait {
    NotBusy,
    Ready,
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    Timeout(Wait),
    Device { status: u8, error: u8 },
    // past the end of the drive, or beyond LBA28 for now
    OutOfRange,
}

pub struct Info {
    pub model: AtaString<40>,
    pub firmware: AtaString<8>,
    pub sectors: u64,
    // only needed beyond LBA28's 2^28 - 1 sectors
    pub lba48: bool,
    pub lba48_supported: bool,
    pub sector_size: u32,
}

impl Info {
    // decodes the 256 words IDENTIFY returns
    pub fn parse(words: &[u16; 256]) -> Info {
        let model = AtaString::parse(&words[27..47]);
        let firmware = AtaString::parse(&words[23..27]);

        let lba28 = u64::from(words[60]) | u64::from(words[61]) << 16;
        let lba48_supported = words[83] & 0x0400 != 0;
        // drives report the LBA28 maximum when they are bigger than that
        let lba48 = lba48_supported && lba28 == 0x0FFF_FFFF;
        let sectors = if lba48 {
            words[100..104].iter().rev().fold(0, |sectors, &word| sectors << 16 | u64::from(word))
        } else {
            lba28
        };

        // word 106 bits 14 and 12: the logical sector size in words 117-118, in 16-bit words
        let sector_size = if words[106] & 0xD000 == 0x5000 {
            (u32::from(words[117]) | u32::from(words[118]) << 16) * 2
        } else {
            512
        };

        Info { model, firmware, sectors, lba48, lba48_supported, sector_size }
    }
}

// text from IDENTIFY, padded with spaces
pub struct AtaString<const N: usize>([u8; N]);

impl<const N: usize> AtaString<N> {
    // ATA strings keep the first of each two characters in the word's high byte
    fn parse(words: &[u16]) -> Self {
        let mut bytes = [0; N];
        for (pair, &word) in bytes.chunks_exact_mut(2).zip(words) {
            pair.copy_from_slice(&word.to_be_bytes());
        }
        Self(bytes)
    }
}

impl<const N: usize> fmt::Display for AtaString<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(core::str::from_utf8(&self.0).unwrap_or("?").trim())
    }
}

pub struct Ata {
    info: Info,
}

impl Ata {
    pub fn info(&self) -> &Info {
        &self.info
    }
}

impl BlockDevice for Ata {
    type Error = Error;

    fn sectors(&self) -> u64 {
        self.info.sectors
    }

    fn read(&mut self, lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), Error> {
        let end = lba + buffers.len() as u64;
        if end > self.info.sectors || end > 1 << 28 {
            return Err(Error::OutOfRange);
        }
        for (index, chunk) in buffers.chunks_mut(MAX_SECTORS).enumerate() {
            read_sectors(lba + (index * MAX_SECTORS) as u64, chunk)?;
        }
        Ok(())
    }
}

pub fn init() -> Result<Ata, Error> {
    power_on();
    init_controller();
    soft_reset()?;
    let info = identify()?;
    Ok(Ata { info })
}

// also when the bootloader already powered it, so nothing depends on the state it left
fn power_on() {
    let was_off = unsafe { GPO32_VAL.read() } & IDE_POWER_OFF != 0;
    unsafe { GPO32_VAL.modify(|value| value & !IDE_POWER_OFF) };
    timer::delay_ms(4);
    unsafe {
        DEV_EN.modify(|value| value | DEV_IDE0);
        GPIOG_ENABLE.write(0);
        GPIOH_ENABLE.write(0);
        gpio_clear(&GPIOI_ENABLE, 0xBF);
        gpio_clear(&GPIOK_ENABLE, 0x1F);
    }
    timer::delay_us(10);
    if was_off {
        // let the voltage build up
        timer::delay_ms(250);
    }
}

fn init_controller() {
    unsafe {
        // bit 5 as in ipodlinux's ipod-ide.c. bit 28 stays off below 65 MHz
        IDE0_CFG.modify(|value| (value | 0x20) & !0x1000_0000);
        IDE0_PRI_TIMING0.write(0xC293);
        IDE0_PRI_TIMING1.write(0x8000_2150);
    }
}

fn soft_reset() -> Result<(), Error> {
    unsafe {
        ATA_SELECT.write(SELECT_LBA);
        ATA_CONTROL.write(CONTROL_NIEN | CONTROL_SRST);
    }
    timer::delay_us(5);
    // interrupts stay off: the driver polls
    unsafe { ATA_CONTROL.write(CONTROL_NIEN) };
    timer::delay_ms(2);
    wait_ready()
}

fn identify() -> Result<Info, Error> {
    // the first and only drive
    unsafe { ATA_SELECT.write(0) };
    wait_ready()?;
    command(CMD_IDENTIFY);
    wait_data()?;

    let mut words = [0; 256];
    for word in &mut words {
        *word = unsafe { ATA_DATA.read() };
    }
    Ok(Info::parse(&words))
}

// one command with LBA28 addressing: the drive asks for each sector's transfer separately
fn read_sectors(lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), Error> {
    unsafe { ATA_SELECT.write(SELECT_LBA) };
    wait_ready()?;
    unsafe {
        // 0 means 256
        ATA_NSECTOR.write(buffers.len() as u8);
        ATA_LBA_LOW.write(lba as u8);
        ATA_LBA_MID.write((lba >> 8) as u8);
        ATA_LBA_HIGH.write((lba >> 16) as u8);
        // bits 24-27 of the sector go in the low half of the select register
        ATA_SELECT.write(SELECT_LBA | (lba >> 24) as u8 & 0x0F);
    }
    command(CMD_READ_SECTORS);

    for buffer in buffers {
        wait_data()?;
        for pair in buffer.0.chunks_exact_mut(2) {
            pair.copy_from_slice(&unsafe { ATA_DATA.read() }.to_le_bytes());
        }
    }

    // after the last sector the drive must be ready, with nothing left to transfer
    wait_not_busy()?;
    let status = alt_status();
    if status & (STATUS_RDY | STATUS_DF | STATUS_DRQ | STATUS_ERR) != STATUS_RDY {
        let error = unsafe { ATA_ERROR.read() };
        return Err(Error::Device { status, error });
    }
    Ok(())
}

fn command(command: u8) {
    unsafe { ATA_COMMAND.write(command) };
    // the status is only valid 400 ns after a command
    timer::delay_us(1);
}

// reading the status acknowledges a pending interrupt, reading the alternate status doesn't
fn status() -> u8 {
    unsafe { ATA_COMMAND.read() }
}

fn alt_status() -> u8 {
    unsafe { ATA_CONTROL.read() }
}

fn wait_not_busy() -> Result<(), Error> {
    wait_until(30_000_000, Wait::NotBusy, || status() & STATUS_BSY == 0)
}

fn wait_ready() -> Result<(), Error> {
    wait_not_busy()?;
    wait_until(10_000_000, Wait::Ready, || alt_status() & STATUS_RDY != 0)
}

// after a command that returns data: the drive must ask for the transfer
fn wait_data() -> Result<(), Error> {
    wait_not_busy()?;
    let status = alt_status();
    if status & (STATUS_ERR | STATUS_DF | STATUS_DRQ) != STATUS_DRQ {
        let error = unsafe { ATA_ERROR.read() };
        return Err(Error::Device { status, error });
    }
    Ok(())
}

fn wait_until(timeout_us: u32, wait: Wait, done: impl Fn() -> bool) -> Result<(), Error> {
    let deadline = timer::micros().wrapping_add(timeout_us);
    while !done() {
        if timer::micros().wrapping_sub(deadline) as i32 >= 0 {
            return Err(Error::Timeout(wait));
        }
    }
    Ok(())
}
