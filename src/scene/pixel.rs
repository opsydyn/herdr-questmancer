#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Self = Self::new(0, 0, 0);

    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

impl PixelPoint {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelSize {
    pub width: u16,
    pub height: u16,
}

impl PixelSize {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl PixelRect {
    #[must_use]
    pub const fn new(x: i32, y: i32, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RgbBuffer {
    size: PixelSize,
    pixels: Vec<Rgb>,
}

impl RgbBuffer {
    #[must_use]
    pub fn filled(width: u16, height: u16, colour: Rgb) -> Self {
        let size = PixelSize::new(width, height);
        let length = pixel_count(size);
        Self {
            size,
            pixels: vec![colour; length],
        }
    }

    #[must_use]
    pub const fn size(&self) -> PixelSize {
        self.size
    }

    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    pub fn put(&mut self, x: i32, y: i32, colour: Rgb) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = colour;
        }
    }

    pub fn clear(&mut self, colour: Rgb) {
        self.pixels.fill(colour);
    }

    pub fn fill_rect(&mut self, rect: PixelRect, colour: Rgb) {
        let start_x = rect.x.max(0);
        let start_y = rect.y.max(0);
        let end_x = rect
            .x
            .saturating_add(i32::from(rect.width))
            .min(i32::from(self.size.width));
        let end_y = rect
            .y
            .saturating_add(i32::from(rect.height))
            .min(i32::from(self.size.height));

        for y in start_y..end_y.max(start_y) {
            for x in start_x..end_x.max(start_x) {
                self.put(x, y, colour);
            }
        }
    }

    #[must_use]
    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [Rgb] {
        &mut self.pixels
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.pixels.capacity()
    }

    pub fn ensure_size(&mut self, width: u16, height: u16, colour: Rgb) {
        let size = PixelSize::new(width, height);
        let length = pixel_count(size);
        self.size = size;
        self.pixels.resize(length, colour);
        self.clear(colour);
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= i32::from(self.size.width) || y >= i32::from(self.size.height) {
            return None;
        }

        let x = usize::try_from(x).expect("non-negative x coordinate fits usize");
        let y = usize::try_from(y).expect("non-negative y coordinate fits usize");
        Some(y * usize::from(self.size.width) + x)
    }
}

fn pixel_count(size: PixelSize) -> usize {
    usize::from(size.width)
        .checked_mul(usize::from(size.height))
        .expect("RGB buffer dimensions overflow pixel allocation")
}
