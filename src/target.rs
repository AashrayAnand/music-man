use std::path::Path;

use crate::{AudioInfo, audio::{AudioError, AudioLocation}, device::AttachedDevice};

// TRAIT: AudioTarget, e.g. an attached drive, the local file cache etc.
// AudioTarget impls are able to be written to, and can be used as a target for exporting audio from an AudioSource:
// 1. contains -> Look for existing AudioInfo in the target.
// 2. 
pub trait AudioTarget {
    fn name(&self) -> &str;
    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError>;
    fn import(&self, source: &Path, info: &AudioInfo) -> Result<AudioLocation, AudioError>;
}

impl AudioTarget for AttachedDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn contains(&self, info: &AudioInfo) -> Result<&AudioLocation, AudioError> {
        self.search(info)
    }
    
    fn import(&self, source: &Path, info: &AudioInfo) -> Result<AudioLocation, AudioError> {
        todo!()
    }
}