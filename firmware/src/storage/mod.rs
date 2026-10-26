use fat::{Cache, Volume};

use crate::drivers::ata::{self, Ata};
use crate::sync::singleton::Singleton;

// too big for the stack
static CACHE: Singleton<Cache> = Singleton::new(Cache::new());

#[derive(Debug)]
pub enum Error {
    Ata(ata::Error),
    Fat(fat::Error<ata::Error>),
    // mount was called before
    Mounted,
}

// powers up the drive and mounts its FAT32 volume, once
pub fn mount() -> Result<Volume<'static, Ata>, Error> {
    let cache = CACHE.take().ok_or(Error::Mounted)?;
    let ata = ata::init().map_err(Error::Ata)?;
    Volume::mount(ata, cache).map_err(Error::Fat)
}
