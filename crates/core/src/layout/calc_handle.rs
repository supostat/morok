use std::ptr;

use morok_style::Side;

use crate::layout::length_slot::LengthSlot;
use crate::layout::track_bound::TrackBound;
use crate::layout::track_list::TrackList;
use crate::layout::track_slot::TrackSlot;

/// Taffy keeps the low bits of a `calc()` handle for its own tag.
const TAG_BITS: u32 = 3;
const SLOT_CODE_BITS: u32 = 5;
const TRACK_ENTRY_BITS: u32 = 12;
const REPEATED_TRACK_BITS: u32 = 10;
const TRACK_BOUND_BITS: u32 = 2;

const SLOT_CODE_SHIFT: u32 = TAG_BITS;
const TRACK_ENTRY_SHIFT: u32 = SLOT_CODE_SHIFT + SLOT_CODE_BITS;
const REPEATED_TRACK_SHIFT: u32 = TRACK_ENTRY_SHIFT + TRACK_ENTRY_BITS;
const TRACK_BOUND_SHIFT: u32 = REPEATED_TRACK_SHIFT + REPEATED_TRACK_BITS;
const ELEMENT_INDEX_SHIFT: u32 = TRACK_BOUND_SHIFT + TRACK_BOUND_BITS;

const PLAIN_SLOTS: [LengthSlot; 21] = [
    LengthSlot::Width,
    LengthSlot::Height,
    LengthSlot::MinWidth,
    LengthSlot::MinHeight,
    LengthSlot::MaxWidth,
    LengthSlot::MaxHeight,
    LengthSlot::Margin(Side::Top),
    LengthSlot::Margin(Side::Right),
    LengthSlot::Margin(Side::Bottom),
    LengthSlot::Margin(Side::Left),
    LengthSlot::Padding(Side::Top),
    LengthSlot::Padding(Side::Right),
    LengthSlot::Padding(Side::Bottom),
    LengthSlot::Padding(Side::Left),
    LengthSlot::Inset(Side::Top),
    LengthSlot::Inset(Side::Right),
    LengthSlot::Inset(Side::Bottom),
    LengthSlot::Inset(Side::Left),
    LengthSlot::FlexBasis,
    LengthSlot::RowGap,
    LengthSlot::ColumnGap,
];

/// The first code is one, so that no handle is the null pointer Taffy rejects.
const FIRST_SLOT_CODE: u64 = 1;
const FIRST_TRACK_LIST_CODE: u64 = FIRST_SLOT_CODE + PLAIN_SLOTS.len() as u64;

/// Names one length of one element.
///
/// Taffy carries a length that mixes pixels with a percentage as an opaque pointer-sized handle
/// and asks the tree for its value once the percentage basis is known. The handle is the place of
/// the length, never an address: nothing is dereferenced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CalcHandle {
    /// The slot of the element in its arena.
    pub(crate) element_index: u32,
    pub(crate) slot: LengthSlot,
}

impl CalcHandle {
    pub(crate) fn to_pointer(self) -> *const () {
        let slot_bits = match self.slot {
            LengthSlot::Track(track) => track_slot_bits(track),
            plain => {
                let position = PLAIN_SLOTS
                    .iter()
                    .position(|slot| *slot == plain)
                    .expect("every slot that is not a track is listed");
                (FIRST_SLOT_CODE + position as u64) << SLOT_CODE_SHIFT
            }
        };
        let bits = u64::from(self.element_index) << ELEMENT_INDEX_SHIFT | slot_bits;
        ptr::without_provenance(bits as usize)
    }

    pub(crate) fn from_pointer(pointer: *const ()) -> Self {
        let bits = pointer.addr() as u64;
        let slot_code = field(bits, SLOT_CODE_SHIFT, SLOT_CODE_BITS);
        let slot = if slot_code < FIRST_TRACK_LIST_CODE {
            PLAIN_SLOTS[(slot_code - FIRST_SLOT_CODE) as usize]
        } else {
            LengthSlot::Track(TrackSlot {
                list: TrackList::ALL[(slot_code - FIRST_TRACK_LIST_CODE) as usize],
                entry: field(bits, TRACK_ENTRY_SHIFT, TRACK_ENTRY_BITS) as usize,
                repeated: field(bits, REPEATED_TRACK_SHIFT, REPEATED_TRACK_BITS) as usize,
                bound: TrackBound::ALL[field(bits, TRACK_BOUND_SHIFT, TRACK_BOUND_BITS) as usize],
            })
        };
        Self {
            element_index: (bits >> ELEMENT_INDEX_SHIFT) as u32,
            slot,
        }
    }
}

fn track_slot_bits(track: TrackSlot) -> u64 {
    assert!(
        track.entry < 1 << TRACK_ENTRY_BITS,
        "a list of grid tracks holds fewer than 4096 entries"
    );
    assert!(
        track.repeated < 1 << REPEATED_TRACK_BITS,
        "a repeat() of grid tracks holds fewer than 1024 tracks"
    );
    let list = TrackList::ALL
        .iter()
        .position(|list| *list == track.list)
        .expect("every track list is listed");
    let bound = TrackBound::ALL
        .iter()
        .position(|bound| *bound == track.bound)
        .expect("every track bound is listed");
    (FIRST_TRACK_LIST_CODE + list as u64) << SLOT_CODE_SHIFT
        | (track.entry as u64) << TRACK_ENTRY_SHIFT
        | (track.repeated as u64) << REPEATED_TRACK_SHIFT
        | (bound as u64) << TRACK_BOUND_SHIFT
}

fn field(bits: u64, shift: u32, width: u32) -> u64 {
    (bits >> shift) & ((1 << width) - 1)
}

#[cfg(test)]
mod tests {
    use morok_style::Side;

    use super::{CalcHandle, PLAIN_SLOTS, TAG_BITS};
    use crate::layout::length_slot::LengthSlot;
    use crate::layout::track_bound::TrackBound;
    use crate::layout::track_list::TrackList;
    use crate::layout::track_slot::TrackSlot;

    fn round_trip(handle: CalcHandle) -> CalcHandle {
        CalcHandle::from_pointer(handle.to_pointer())
    }

    #[test]
    fn every_plain_slot_survives_the_round_trip() {
        for slot in PLAIN_SLOTS {
            let handle = CalcHandle {
                element_index: u32::MAX,
                slot,
            };

            assert_eq!(round_trip(handle), handle);
        }
    }

    #[test]
    fn every_track_slot_survives_the_round_trip() {
        for list in TrackList::ALL {
            for bound in TrackBound::ALL {
                let handle = CalcHandle {
                    element_index: 7,
                    slot: LengthSlot::Track(TrackSlot {
                        list,
                        entry: 4095,
                        repeated: 1023,
                        bound,
                    }),
                };

                assert_eq!(round_trip(handle), handle);
            }
        }
    }

    #[test]
    fn a_handle_is_never_null_and_leaves_the_tag_bits_of_taffy_clear() {
        let first = CalcHandle {
            element_index: 0,
            slot: LengthSlot::Width,
        };
        let sided = CalcHandle {
            element_index: 0,
            slot: LengthSlot::Inset(Side::Left),
        };

        for handle in [first, sided] {
            let address = handle.to_pointer().addr();

            assert_ne!(address, 0);
            assert_eq!(address & ((1 << TAG_BITS) - 1), 0);
        }
    }

    #[test]
    #[should_panic(expected = "fewer than 4096 entries")]
    fn a_track_entry_past_the_handle_capacity_panics() {
        let handle = CalcHandle {
            element_index: 0,
            slot: LengthSlot::Track(TrackSlot {
                list: TrackList::TemplateColumns,
                entry: 4096,
                repeated: 0,
                bound: TrackBound::Minimum,
            }),
        };

        let _ = handle.to_pointer();
    }
}
