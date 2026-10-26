use crate::block::{BlockDevice, SECTOR_SIZE, SectorBuffer};
use crate::cache::Cache;
use crate::mbr::PartitionTable;

const SIGNATURE: u16 = 0xAA55;
// fewer clusters than this is FAT12 or FAT16, by definition
const MIN_FAT32_CLUSTERS: u64 = 65_525;
const MAX_CLUSTER_BYTES: u32 = 128 * 1024;
const FSINFO_LEAD: u32 = 0x4161_5252;
const FSINFO_STRUCT: u32 = 0x6141_7272;
const FSINFO_UNKNOWN: u32 = 0xFFFF_FFFF;

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Device(E),
    // sector 0 holds neither a partition table nor a boot sector
    NoSignature,
    Gpt,
    // only FAT12 or FAT16 volumes were found
    NotFat32,
    NoVolume,
}

// a mounted FAT32 file system: the device plus where everything on it lives
// counts are in 512-byte disk sectors, whatever the FAT's own sector size
pub struct Volume<'a, D> {
    device: D,
    cache: &'a mut Cache,
    // first sector of the volume on the disk
    start: u64,
    sectors_per_cluster: u32,
    // the first FAT, the map of which cluster follows which
    fat_start: u64,
    fat_sectors: u64,
    fats: u8,
    // cluster 2, where the contents of files and folders begin
    data_start: u64,
    clusters: u32,
    root_cluster: u32,
    free_clusters: Option<u32>,
    volume_id: u32,
    label: [u8; 11],
}

impl<'a, D: BlockDevice> Volume<'a, D> {
    // a boot sector at sector 0 (a "superfloppy"), else the first partition holding one
    pub fn mount(mut device: D, cache: &'a mut Cache) -> Result<Self, Error<D::Error>> {
        let mut sector = SectorBuffer::new();
        read(&mut device, 0, &mut sector)?;
        if sector.u16(510) != SIGNATURE {
            return Err(Error::NoSignature);
        }

        let mut not_fat32 = false;
        match BootSector::parse(&sector) {
            Ok(boot) => return Self::from_boot(device, cache, 0, boot),
            Err(Parse::NotFat32) => not_fat32 = true,
            Err(Parse::Invalid) => {}
        }

        let table = PartitionTable::parse(&sector).ok_or(Error::Gpt)?;
        for partition in table.partitions() {
            for lba in partition.candidates(device.sectors()) {
                read(&mut device, lba, &mut sector)?;
                match BootSector::parse(&sector) {
                    Ok(boot) => return Self::from_boot(device, cache, lba, boot),
                    Err(Parse::NotFat32) => not_fat32 = true,
                    Err(Parse::Invalid) => {}
                }
            }
        }
        Err(if not_fat32 { Error::NotFat32 } else { Error::NoVolume })
    }

    // the volume whose boot sector is at lba
    pub fn open(mut device: D, cache: &'a mut Cache, lba: u64) -> Result<Self, Error<D::Error>> {
        let mut sector = SectorBuffer::new();
        read(&mut device, lba, &mut sector)?;
        match BootSector::parse(&sector) {
            Ok(boot) => Self::from_boot(device, cache, lba, boot),
            Err(Parse::NotFat32) => Err(Error::NotFat32),
            Err(Parse::Invalid) => Err(Error::NoVolume),
        }
    }

    // converts the boot sector to disk sectors and reads the free count from FSInfo
    fn from_boot(mut device: D, cache: &'a mut Cache, start: u64, boot: BootSector) -> Result<Self, Error<D::Error>> {
        // it may hold sectors of another device
        cache.clear();
        let scale = boot.bytes_per_sector / SECTOR_SIZE as u32;
        let fat_start = start + u64::from(boot.reserved * scale);
        let fat_sectors = u64::from(boot.fat_size) * u64::from(scale);

        let mut free_clusters = None;
        if boot.fsinfo != 0 && boot.fsinfo != 0xFFFF {
            let lba = start + u64::from(boot.fsinfo * scale);
            let sector = cache.sector(&mut device, lba).map_err(Error::Device)?;
            let free = sector.u32(488);
            if sector.u32(0) == FSINFO_LEAD
                && sector.u32(484) == FSINFO_STRUCT
                && free != FSINFO_UNKNOWN
                && free <= boot.clusters
            {
                free_clusters = Some(free);
            }
        }

        Ok(Volume {
            device,
            cache,
            start,
            sectors_per_cluster: boot.sectors_per_cluster * scale,
            fat_start,
            fat_sectors,
            fats: boot.fats,
            data_start: fat_start + u64::from(boot.fats) * fat_sectors,
            clusters: boot.clusters,
            root_cluster: boot.root_cluster,
            free_clusters,
            volume_id: boot.volume_id,
            label: boot.label,
        })
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    // in bytes
    pub fn cluster_size(&self) -> u32 {
        self.sectors_per_cluster * SECTOR_SIZE as u32
    }

    pub fn clusters(&self) -> u32 {
        self.clusters
    }

    // from FSInfo, which may be missing or out of date
    pub fn free_clusters(&self) -> Option<u32> {
        self.free_clusters
    }

    pub fn volume_id(&self) -> u32 {
        self.volume_id
    }

    // padded with spaces
    pub fn label(&self) -> &[u8; 11] {
        &self.label
    }

    pub fn into_device(self) -> D {
        self.device
    }
}

fn read<D: BlockDevice>(device: &mut D, lba: u64, sector: &mut SectorBuffer) -> Result<(), Error<D::Error>> {
    device.read(lba, core::slice::from_mut(sector)).map_err(Error::Device)
}

enum Parse {
    Invalid,
    NotFat32,
}

// the first sector of a volume, describing its layout
// counts in the FAT's own sectors
struct BootSector {
    bytes_per_sector: u32,
    sectors_per_cluster: u32,
    reserved: u32,
    fats: u8,
    fat_size: u32,
    clusters: u32,
    root_cluster: u32,
    fsinfo: u32,
    volume_id: u32,
    label: [u8; 11],
}

impl BootSector {
    // offsets are fixed by the FAT32 spec, from the start of the boot sector
    fn parse(sector: &SectorBuffer) -> Result<Self, Parse> {
        let bytes_per_sector = u32::from(sector.u16(11));
        let sectors_per_cluster = u32::from(sector.0[13]);
        let reserved = u32::from(sector.u16(14));
        let fats = sector.0[16];
        let total = match sector.u16(19) {
            0 => sector.u32(32),
            total => u32::from(total),
        };
        let fat_size = match sector.u16(22) {
            0 => sector.u32(36),
            size => u32::from(size),
        };

        let sane = sector.u16(510) == SIGNATURE
            && matches!(bytes_per_sector, 512 | 1024 | 2048 | 4096)
            && sectors_per_cluster.is_power_of_two()
            && bytes_per_sector * sectors_per_cluster <= MAX_CLUSTER_BYTES
            && reserved != 0
            && fats != 0
            && fat_size != 0;
        if !sane {
            return Err(Parse::Invalid);
        }

        // FAT12 and FAT16 also have a fixed root directory region, FAT32 has none
        let root_dir = (u64::from(sector.u16(17)) * 32).div_ceil(u64::from(bytes_per_sector));
        let metadata = u64::from(reserved) + u64::from(fats) * u64::from(fat_size) + root_dir;
        let Some(data) = u64::from(total).checked_sub(metadata) else {
            return Err(Parse::Invalid);
        };
        let clusters = data / u64::from(sectors_per_cluster);
        if clusters < MIN_FAT32_CLUSTERS {
            return Err(Parse::NotFat32);
        }

        Ok(BootSector {
            bytes_per_sector,
            sectors_per_cluster,
            reserved,
            fats,
            fat_size,
            clusters: clusters as u32,
            root_cluster: sector.u32(44),
            fsinfo: u32::from(sector.u16(48)),
            volume_id: sector.u32(67),
            label: sector.0[71..82].try_into().unwrap(),
        })
    }
}
