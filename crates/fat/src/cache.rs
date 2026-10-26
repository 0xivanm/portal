use crate::block::{BlockDevice, SectorBuffer};

// disk sectors per line: one 2048-byte FAT sector on an iPod 5.5G
pub const LINE_SECTORS: usize = 4;
pub const LINES: usize = 16;

struct Line {
    start: u64,
    // sectors that hold data, 0 while empty
    count: usize,
    // the clock at its last use, the lowest is evicted first
    used: u32,
    sectors: [SectorBuffer; LINE_SECTORS],
}

impl Line {
    const EMPTY: Line = Line { start: 0, count: 0, used: 0, sectors: [const { SectorBuffer::new() }; LINE_SECTORS] };

    fn holds(&self, lba: u64) -> bool {
        lba >= self.start && lba - self.start < self.count as u64
    }
}

// recently read FAT and directory sectors, file contents go past it
pub struct Cache {
    lines: [Line; LINES],
    // counts lookups, ages are wrapping differences from it
    clock: u32,
}

impl Cache {
    pub const fn new() -> Self {
        Self { lines: [Line::EMPTY; LINES], clock: 0 }
    }

    pub(crate) fn clear(&mut self) {
        for line in &mut self.lines {
            line.count = 0;
        }
    }

    // the sector at lba, read with the rest of its line on a miss
    pub(crate) fn sector<D: BlockDevice>(&mut self, device: &mut D, lba: u64) -> Result<&SectorBuffer, D::Error> {
        self.clock = self.clock.wrapping_add(1);
        let index = match self.lines.iter().position(|line| line.holds(lba)) {
            Some(index) => index,
            None => {
                let index = self.victim();
                let line = &mut self.lines[index];
                // a line that would pass the end of the disk reads just this sector,
                // so the device still reports a sector that is out of range
                let aligned = lba - lba % LINE_SECTORS as u64;
                (line.start, line.count) = if aligned + LINE_SECTORS as u64 <= device.sectors() {
                    (aligned, LINE_SECTORS)
                } else {
                    (lba, 1)
                };
                if let Err(error) = device.read(line.start, &mut line.sectors[..line.count]) {
                    line.count = 0;
                    return Err(error);
                }
                index
            }
        };
        let line = &mut self.lines[index];
        line.used = self.clock;
        Ok(&line.sectors[(lba - line.start) as usize])
    }

    // an empty line, else the least recently used
    fn victim(&self) -> usize {
        let age = |line: &Line| if line.count == 0 { u32::MAX } else { self.clock.wrapping_sub(line.used) };
        (0..LINES).max_by_key(|&index| age(&self.lines[index])).unwrap()
    }
}

