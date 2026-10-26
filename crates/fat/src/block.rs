pub const SECTOR_SIZE: usize = 512;

// 16-byte aligned, a cache line, so a device can DMA into it directly
#[repr(C, align(16))]
pub struct SectorBuffer(pub [u8; SECTOR_SIZE]);

impl SectorBuffer {
    pub const fn new() -> Self {
        Self([0; SECTOR_SIZE])
    }

    // little-endian, built from bytes since many FAT fields are unaligned
    pub fn u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.0[offset], self.0[offset + 1]])
    }

    pub fn u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.0[offset..offset + 4].try_into().unwrap())
    }
}

// a disk as numbered 512-byte sectors, all the fat crate needs from the hardware
// implemented by the ATA driver on the iPod and by a disk image in tests
pub trait BlockDevice {
    type Error;

    fn sectors(&self) -> u64;

    // fills the buffers from consecutive sectors, starting at lba
    fn read(&mut self, lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), Self::Error>;
}

// impl<D: BlockDevice> BlockDevice for &mut D {
//     type Error = D::Error;

//     fn sectors(&self) -> u64 {
//         (**self).sectors()
//     }

//     fn read(&mut self, lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), Self::Error> {
//         (**self).read(lba, buffers)
//     }
// }
