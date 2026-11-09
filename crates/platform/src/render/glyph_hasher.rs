use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

use crate::render::atlas_key::AtlasKey;

/// The multiplier of the Fx hash: a fast hash for small keys of integers, which glyph keys are.
const MULTIPLIER: u64 = 0x517c_c1b7_2722_0a95;
const ROTATION: u32 = 5;

/// Hashes the integer fields of an atlas key with a few multiplications: the keys come from
/// fonts and the application, not from a stranger, so the hash need not defend against them.
#[derive(Debug, Default)]
pub(crate) struct GlyphHasher(u64);

pub(crate) type GlyphMap<Value> = HashMap<AtlasKey, Value, BuildHasherDefault<GlyphHasher>>;
pub(crate) type GlyphSet = HashSet<AtlasKey, BuildHasherDefault<GlyphHasher>>;

impl GlyphHasher {
    fn mix(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(ROTATION) ^ word).wrapping_mul(MULTIPLIER);
    }
}

impl Hasher for GlyphHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.mix(u64::from(*byte));
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.mix(u64::from(value));
    }

    fn write_u16(&mut self, value: u16) {
        self.mix(u64::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.mix(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.mix(value);
    }

    fn write_usize(&mut self, value: usize) {
        self.mix(value as u64);
    }
}
