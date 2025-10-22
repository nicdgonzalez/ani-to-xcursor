use bitflags::bitflags;

bitflags! {
    /// Represents a bit flag used in the ANI header.
    #[derive(Debug, Clone, Copy)]
    pub struct Flag: u32 {
        /// Indicates the frames are in Windows ICO format.
        const ICON = 0x01;
        /// Indicates the animation has a custom sequence.
        ///
        /// Custom sequences are commonly used to save space and avoid repeating frames.
        const SEQUENCE = 0x02;
    }
}

/// Represents the `anih` chunk of an ANI file.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Header {
    size: u32,
    frames: u32,
    steps: u32,

    // Not used.
    x: u32,
    y: u32,
    bit_count: u32,
    planes: u32,

    jif_rate: u32,
    flags: Flag,
}

impl Header {
    /// The length of the ANI header (should always be 36).
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// The number of frames we can expect to find in the `fram` chunk.
    pub const fn frames(&self) -> u32 {
        self.frames
    }

    /// The number of steps in the animation loop.
    pub const fn steps(&self) -> u32 {
        self.steps
    }

    /// The default display rate in, jiffies (1/60 seconds).
    pub const fn jif_rate(&self) -> u32 {
        self.jif_rate
    }

    /// Bit flags.
    pub const fn flags(&self) -> &Flag {
        &self.flags
    }
}
