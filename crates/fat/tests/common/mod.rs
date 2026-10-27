#![allow(dead_code)]

use std::io::Cursor;

use fat::{BlockDevice, SECTOR_SIZE, SectorBuffer};
use fatfs::{FatType, FormatVolumeOptions};

// a disk image in memory
pub struct Image(pub Vec<u8>);

impl BlockDevice for Image {
    type Error = ();

    fn sectors(&self) -> u64 {
        (self.0.len() / SECTOR_SIZE) as u64
    }

    fn read(&mut self, lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), ()> {
        for (index, buffer) in buffers.iter_mut().enumerate() {
            let at = (lba as usize + index) * SECTOR_SIZE;
            buffer.0.copy_from_slice(self.0.get(at..at + SECTOR_SIZE).ok_or(())?);
        }
        Ok(())
    }
}

pub const LABEL: [u8; 11] = *b"PORTAL TEST";
pub const VOLUME_ID: u32 = 0x1234_5678;

// formats bytes [offset, offset + size) of the image with fatfs, the reference implementation
pub fn format(image: &mut [u8], offset: usize, size: usize, fat_type: FatType, bytes_per_sector: u16) {
    let options = FormatVolumeOptions::new()
        .fat_type(fat_type)
        .bytes_per_sector(bytes_per_sector)
        .bytes_per_cluster(u32::from(bytes_per_sector))
        .volume_label(LABEL)
        .volume_id(VOLUME_ID);
    fatfs::format_volume(&mut Cursor::new(&mut image[offset..offset + size]), options).unwrap();
}

// what fatfs itself counts as free on the volume at offset
pub fn fatfs_free(image: &mut [u8], offset: usize, size: usize) -> u32 {
    let fs = fatfs::FileSystem::new(Cursor::new(&mut image[offset..offset + size]), fatfs::FsOptions::new()).unwrap();
    fs.stats().unwrap().free_clusters()
}

// an MBR with (type, start, size) entries, in whatever unit the caller uses
pub fn partition_table(image: &mut [u8], entries: &[(u8, u32, u32)]) {
    for (index, &(kind, start, size)) in entries.iter().enumerate() {
        let entry = 446 + index * 16;
        image[entry + 4] = kind;
        image[entry + 8..entry + 12].copy_from_slice(&start.to_le_bytes());
        image[entry + 12..entry + 16].copy_from_slice(&size.to_le_bytes());
    }
    image[510] = 0x55;
    image[511] = 0xAA;
}

// big enough for FAT32's minimum of 65,525 clusters with one sector per cluster
pub fn fat32_size(bytes_per_sector: usize) -> usize {
    66_000 * bytes_per_sector + 2 * 1024 * 1024
}
