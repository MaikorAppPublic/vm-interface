pub struct LayerHeader {
    pub is_visible: bool,
    pub x_offset: u8,
    pub y_offset: u8
}

impl LayerHeader {
    pub fn new(data: &[u8]) -> Self {
        Self { is_visible: data[2] & 0b1 == 0b1, x_offset: data[0], y_offset: data[1] }
    }
}

#[derive(Debug)]
pub struct Sprite {
    pub x: usize,
    pub y: usize,
    pub id: usize,
    pub flipped_vert: bool,
    pub flipped_horz: bool,
    pub palette: usize,
    pub large: bool,
    pub order: usize
}

impl Sprite {
    pub(crate) fn new(data: &[u8]) -> Self {
        Self {
            x: data[0] as usize,
            y: data[1] as usize,
            id: u16::from_be_bytes([data[2], data[3] & 0x80]) as usize,
            flipped_vert: data[3] & 0x40 == 0x40,
            flipped_horz: data[3] & 0x20 == 0x20,
            palette: (data[3] & 0x18) as usize,
            large: data[3] & 0x4 == 0x4,
            order: (data[3] & 0x3) as usize,
        }
    }
}