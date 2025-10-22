use std::{error, fmt, mem, slice};

/// Reinterpret `&T` as `&[u8]`.
fn as_bytes<T: Copy>(value: &T) -> &[u8] {
    let data = slice::from_ref(value);
    let new_length = mem::size_of::<T>() / mem::size_of::<u8>();
    assert_eq!((data.as_ptr() as usize) % mem::size_of::<u8>(), 0);
    // SAFETY: Casting to bytes is the safest type of cast.
    unsafe { slice::from_raw_parts(data.as_ptr().cast::<u8>(), new_length) }
}

/// Represents a frame of the cursor animation.
///
/// A frame may contain one or more images in ICO format.
#[derive(Debug, Clone)]
pub struct Frame {
    header: IconDir,
    images: Vec<Image>,
}

impl Frame {
    pub const fn new(header: IconDir, images: Vec<Image>) -> Self {
        Self { header, images }
    }

    /// Contains information about the images stored within this frame.
    pub const fn header(&self) -> IconDir {
        self.header
    }

    /// A collection of images stored within this frame.
    pub fn images(&self) -> &[Image] {
        &self.images
    }

    /// Copies the bytes of `self` into a new `Vec`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(as_bytes(&self.header));

        for image in &self.images {
            bytes.extend(as_bytes(&image.header));
            bytes.extend(&image.data);
        }

        bytes
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IconDir {
    reserved: u16,
    image_type: u16,
    image_count: u16,
}

impl IconDir {
    pub const fn new(image_type: ImageType, image_count: u16) -> Self {
        Self {
            reserved: 0,
            image_type: image_type as u16,
            image_count,
        }
    }

    /// Indicates which file format the images in this directory are stored in.
    pub const fn image_type(self) -> u16 {
        self.image_type
    }

    /// Indicates how many images are stored within this directory.
    pub const fn image_count(self) -> u16 {
        self.image_count
    }
}

/// Indicates which file format the images are in.
#[repr(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    /// Image is in ICO format.
    Ico = 1,
    /// Image is in CUR format.
    Cur = 2,
}

impl TryFrom<u16> for ImageType {
    type Error = ImageTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Ico),
            2 => Ok(Self::Cur),
            _ => Err(ImageTypeError),
        }
    }
}

/// Indicates an invalid value was used when trying to convert a `u16` into an [`ImageType`].
#[derive(Debug)]
pub struct ImageTypeError;

impl fmt::Display for ImageTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "invalid image type".fmt(f)
    }
}

impl error::Error for ImageTypeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Represents an image in a [`Frame`].
#[derive(Debug, Clone)]
pub struct Image {
    header: IconDirEntry,
    data: Vec<u8>,
}

impl Image {
    pub const fn new(header: IconDirEntry, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    /// Contains metadata about the image.
    pub const fn header(&self) -> &IconDirEntry {
        &self.header
    }

    /// The image data itself, minus the header.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Contains information about the image in an [`IconDir`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IconDirEntry {
    width: u8,
    height: u8,
    colors: u8,

    // Not used.
    reserved: u8,

    color_planes_or_hotspot_x: u16,
    bits_per_pixel_or_hotspot_y: u16,
    data_size: u32,
    data_offset: u32,
}

impl IconDirEntry {
    /// The width of the image in pixels.
    pub fn width(&self) -> u16 {
        match self.width {
            0 => 256,
            w => u16::from(w),
        }
    }

    /// The height of the image in pixels.
    pub fn height(&self) -> u16 {
        match self.height {
            0 => 256,
            h => u16::from(h),
        }
    }

    /// The number of colors in the color palette (or 0 if no color palette is used).
    pub const fn colors(&self) -> u8 {
        self.colors
    }

    /// This specifies color planes (should be 0 or 1).
    ///
    /// This value's meaning changes depending if the image is in ICO or CUR format.
    ///
    /// This is an alias for [`Self::hotspot_x`].
    ///
    /// If CUR: This specifies the number of pixels to the tip of the cursor from the left.
    pub const fn color_planes(&self) -> u16 {
        self.color_planes_or_hotspot_x
    }

    /// This specifies the number of pixels to the tip of the cursor from the left.
    ///
    /// This value's meaning changes depending if the image is in ICO or CUR format.
    ///
    /// This is an alias for [`Self::color_planes`].
    ///
    /// If ICO: This specifies color planes (should be 0 or 1).
    pub const fn hotspot_x(&self) -> u16 {
        self.color_planes_or_hotspot_x
    }

    /// This specifies the number of bits per pixel.
    ///
    /// This value's meaning changes depending if the image is in ICO or CUR format.
    ///
    /// This is an alias for [`Self::hotspot_y`].
    ///
    /// If CUR: This specifies the number of pixels to the tip of the cursor from the top.
    pub const fn bits_per_pixel(&self) -> u16 {
        self.bits_per_pixel_or_hotspot_y
    }

    /// This specifies the number of pixels to the tip of the cursor from the top.
    ///
    /// This value's meaning changes depending if the image is in ICO or CUR format.
    ///
    /// This is an alias for [`Self::bits_per_pixel`].
    ///
    /// If ICO: This specifies the number of bits per pixel.
    pub const fn hotspot_y(&self) -> u16 {
        self.bits_per_pixel_or_hotspot_y
    }

    /// The size of the image's data in bytes.
    pub const fn data_size(&self) -> u32 {
        self.data_size
    }

    /// The offset of BMP or PNG data from the beginning of the ICO/CUR file.
    pub const fn data_offset(&self) -> u32 {
        self.data_offset
    }
}
