use crate::block::SectorBuffer;

// four 16-byte entries: type at 4, start at 8, size at 12
const ENTRIES: usize = 446;
// a GPT disk keeps one entry of this type covering the whole disk
const GPT_PROTECTIVE: u8 = 0xEE;
// a chain of more partitions, not supported
const EXTENDED: [u8; 3] = [0x05, 0x0F, 0x85];

// one entry, counted in whatever unit the table uses
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Partition {
    pub kind: u8,
    pub start: u64,
    pub size: u64,
}

impl Partition {
    // the disk sectors its boot sector may be at
    // iPod 5.5Gs formatted over USB count in 2048-byte units, and the table doesn't say which it uses
    pub fn candidates(self, sectors: u64) -> impl Iterator<Item = u64> {
        [1, 2, 4].into_iter().map(move |unit| self.start * unit).take_while(move |&lba| lba < sectors)
    }
}

// the partition table of an MBR in sector 0
pub struct PartitionTable([Partition; 4]);

impl PartitionTable {
    // None on a GPT disk, whose partitions are elsewhere
    pub fn parse(sector: &SectorBuffer) -> Option<Self> {
        let entries: [Partition; 4] = core::array::from_fn(|index| {
            let entry = ENTRIES + index * 16;
            Partition {
                kind: sector.0[entry + 4],
                start: u64::from(sector.u32(entry + 8)),
                size: u64::from(sector.u32(entry + 12)),
            }
        });
        if entries.iter().any(|partition| partition.kind == GPT_PROTECTIVE) {
            return None;
        }
        Some(Self(entries))
    }

    // the entries that may hold a volume
    // Apple's firmware partition has type 0, which normally means empty, so the size decides
    pub fn partitions(&self) -> impl Iterator<Item = Partition> {
        self.0.into_iter().filter(|partition| partition.size != 0 && !EXTENDED.contains(&partition.kind))
    }
}
