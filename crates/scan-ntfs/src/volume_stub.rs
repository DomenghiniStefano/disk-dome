//! Other platforms: there is no `$MFT` to read; opening a volume reports `Unsupported`.

use crate::pipeline::{ChunkReader, ScanError, Source};

pub struct Volume;

pub struct NoReader;

impl ChunkReader for NoReader {
    fn read_at(&mut self, _offset: u64, _buf: &mut [u8]) -> Result<(), ScanError> {
        Err(ScanError::Unsupported)
    }
}

impl Volume {
    pub fn open(_letter: char) -> Result<Self, ScanError> {
        Err(ScanError::Unsupported)
    }

    pub fn summary(&self) -> crate::VolumeSummary {
        crate::VolumeSummary::default()
    }
}

impl Source for Volume {
    type Reader<'a> = NoReader;
    fn open_reader(&self) -> Result<Self::Reader<'_>, ScanError> {
        Err(ScanError::Unsupported)
    }
    fn len(&self) -> u64 {
        0
    }
    fn record_size(&self) -> usize {
        1024
    }
    fn alignment(&self) -> usize {
        1024
    }
}
