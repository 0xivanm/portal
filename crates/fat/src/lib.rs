#![no_std]

mod block;
mod mbr;
mod volume;

pub use block::{BlockDevice, SECTOR_SIZE, SectorBuffer};
pub use mbr::{Partition, PartitionTable};
pub use volume::{Error, Volume};
