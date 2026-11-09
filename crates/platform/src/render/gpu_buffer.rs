use std::marker::PhantomData;

use bytemuck::Pod;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLAllocation, MTLBuffer, MTLDevice, MTLGPUAddress, MTLResidencySet, MTLResourceOptions};

/// A growable array of entries that lives in a Metal buffer: entries are written straight into
/// the memory the GPU reads. The buffer keeps itself in the residency set it was created with.
#[derive(Debug)]
pub(crate) struct GpuBuffer<Entry> {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    residency_set: Retained<ProtocolObject<dyn MTLResidencySet>>,
    buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    /// The memory of the buffer; asking the buffer for it costs a message per entry otherwise.
    entries: *mut Entry,
    capacity: usize,
    length: usize,
    /// Buffers outgrown during the frame: a draw encoded before the growth still reads them,
    /// so they live until the frame is over.
    retired: Vec<Retained<ProtocolObject<dyn MTLBuffer>>>,
    entry: PhantomData<Entry>,
}

impl<Entry: Pod> GpuBuffer<Entry> {
    /// # Panics
    ///
    /// When `capacity` is zero: the GPU needs an address even for an empty array.
    pub(crate) fn with_capacity(
        device: &Retained<ProtocolObject<dyn MTLDevice>>,
        residency_set: &Retained<ProtocolObject<dyn MTLResidencySet>>,
        capacity: usize,
    ) -> Self {
        assert!(capacity > 0, "a GPU buffer holds at least one entry");
        let buffer = allocate::<Entry>(device, capacity);
        residency_set.addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*buffer));
        residency_set.commit();
        Self {
            device: device.clone(),
            residency_set: residency_set.clone(),
            entries: entries_of(&buffer),
            buffer,
            capacity,
            length: 0,
            retired: Vec::new(),
            entry: PhantomData,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.length
    }

    /// Empties the buffer for a new frame; the GPU must have finished the previous one.
    pub(crate) fn clear(&mut self) {
        self.length = 0;
        if self.retired.is_empty() {
            return;
        }
        for retired in self.retired.drain(..) {
            self.residency_set
                .removeAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*retired));
        }
        self.residency_set.commit();
    }

    pub(crate) fn push(&mut self, entry: Entry) {
        if self.length == self.capacity {
            self.grow();
        }
        // SAFETY: the buffer holds `capacity` entries and `length` is below it; its memory is
        // page-aligned CPU memory that only this struct writes.
        unsafe { self.entries.add(self.length).write(entry) };
        self.length += 1;
    }

    /// # Panics
    ///
    /// When no entry was written at `index`.
    pub(crate) fn get(&self, index: usize) -> Entry {
        assert!(
            index < self.length,
            "entry {index} of a buffer of {} entries",
            self.length
        );
        // SAFETY: every entry below `length` was written by `push`.
        unsafe { self.entries.add(index).read() }
    }

    /// The address the shaders read the first entry at.
    pub(crate) fn gpu_address(&self) -> MTLGPUAddress {
        self.buffer.gpuAddress()
    }

    /// The address the shaders read the entry at `index` at.
    ///
    /// # Panics
    ///
    /// When no entry was written at `index`.
    pub(crate) fn gpu_address_of(&self, index: usize) -> MTLGPUAddress {
        assert!(
            index < self.length,
            "entry {index} of a buffer of {} entries",
            self.length
        );
        self.buffer.gpuAddress() + (index * size_of::<Entry>()) as MTLGPUAddress
    }

    fn grow(&mut self) {
        let capacity = self.capacity * 2;
        let buffer = allocate::<Entry>(&self.device, capacity);
        let entries = entries_of::<Entry>(&buffer);
        // SAFETY: both buffers hold at least `length` entries and are separate allocations.
        unsafe { entries.copy_from_nonoverlapping(self.entries, self.length) };
        self.residency_set
            .addAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&*buffer));
        self.residency_set.commit();
        let outgrown = std::mem::replace(&mut self.buffer, buffer);
        self.retired.push(outgrown);
        self.entries = entries;
        self.capacity = capacity;
    }
}

impl<Entry> Drop for GpuBuffer<Entry> {
    fn drop(&mut self) {
        for buffer in self.retired.iter().chain([&self.buffer]) {
            self.residency_set
                .removeAllocation(ProtocolObject::<dyn MTLAllocation>::from_ref(&**buffer));
        }
        self.residency_set.commit();
    }
}

/// The memory of a shared buffer; it stays where it is for as long as the buffer lives.
fn entries_of<Entry>(buffer: &ProtocolObject<dyn MTLBuffer>) -> *mut Entry {
    buffer.contents().as_ptr().cast::<Entry>()
}

fn allocate<Entry>(device: &ProtocolObject<dyn MTLDevice>, capacity: usize) -> Retained<ProtocolObject<dyn MTLBuffer>> {
    let bytes = capacity * size_of::<Entry>();
    device
        .newBufferWithLength_options(bytes, MTLResourceOptions::StorageModeShared)
        .unwrap_or_else(|| panic!("Metal allocates a buffer of {bytes} bytes"))
}
