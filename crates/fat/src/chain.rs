use crate::block::{BlockDevice, SECTOR_SIZE};
use crate::volume::{Error, Volume};

// FAT32 entries are 32 bits, but only the low 28 count
const ENTRY_MASK: u32 = 0x0FFF_FFFF;
const BAD: u32 = 0x0FFF_FFF7;
// anything from here up ends a chain
const END: u32 = 0x0FFF_FFF8;

// what the FAT says about one cluster
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    Free,
    Next(u32),
    Bad,
    End,
}

impl<'a, D: BlockDevice> Volume<'a, D> {
    // the clusters of a file or directory in order, none for a start of 0 (an empty file)
    pub fn chain(&mut self, start: u32) -> Chain<'_, 'a, D> {
        Chain { volume: self, next: (start != 0).then_some(start), steps: 0 }
    }

    // the disk sector where a cluster's data begins
    pub fn cluster_lba(&self, cluster: u32) -> u64 {
        self.data_start + u64::from(cluster - 2) * u64::from(self.sectors_per_cluster)
    }

    pub(crate) fn entry(&mut self, cluster: u32) -> Result<Entry, Error<D::Error>> {
        if !self.is_cluster(cluster) {
            return Err(Error::Corrupt);
        }
        let byte = u64::from(cluster) * 4;
        let lba = self.fat_start + byte / SECTOR_SIZE as u64;
        let sector = self.cache.sector(&mut self.device, lba).map_err(Error::Device)?;
        match sector.u32((byte % SECTOR_SIZE as u64) as usize) & ENTRY_MASK {
            0 => Ok(Entry::Free),
            BAD => Ok(Entry::Bad),
            END.. => Ok(Entry::End),
            next if self.is_cluster(next) => Ok(Entry::Next(next)),
            _ => Err(Error::Corrupt),
        }
    }

    // clusters are numbered from 2, the first two FAT entries are reserved
    fn is_cluster(&self, cluster: u32) -> bool {
        cluster >= 2 && cluster - 2 < self.clusters
    }
}

pub struct Chain<'v, 'a, D> {
    volume: &'v mut Volume<'a, D>,
    next: Option<u32>,
    steps: u32,
}

impl<D: BlockDevice> Iterator for Chain<'_, '_, D> {
    type Item = Result<u32, Error<D::Error>>;

    // stops after the first error
    fn next(&mut self) -> Option<Self::Item> {
        let cluster = self.next.take()?;
        // no chain is longer than the volume, so a longer one loops
        self.steps += 1;
        if self.steps > self.volume.clusters {
            return Some(Err(Error::Corrupt));
        }
        match self.volume.entry(cluster) {
            Ok(Entry::Next(next)) => self.next = Some(next),
            Ok(Entry::End) => {}
            // a chain never runs into a free or bad cluster
            Ok(Entry::Free | Entry::Bad) => return Some(Err(Error::Corrupt)),
            Err(error) => return Some(Err(error)),
        }
        Some(Ok(cluster))
    }
}
