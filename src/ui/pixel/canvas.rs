use super::ColorRole;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Canvas {
    width: u16,
    height: u16,
    pixels: Vec<Option<ColorRole>>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixels: vec![None; usize::from(width) * usize::from(height)],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, role: ColorRole) {
        if let Some(pixel) = self
            .index(x, y)
            .and_then(|index| self.pixels.get_mut(index))
        {
            *pixel = Some(role);
        }
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, role: ColorRole) {
        let x_end = x.saturating_add(width).min(self.width);
        let y_end = y.saturating_add(height).min(self.height);

        for row in y..y_end {
            for column in x..x_end {
                self.set(column, row, role);
            }
        }
    }

    pub fn pixels(&self) -> &[Option<ColorRole>] {
        &self.pixels
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub(super) fn pixel(&self, x: u16, y: u16) -> Option<ColorRole> {
        self.index(x, y)
            .and_then(|index| self.pixels.get(index))
            .copied()
            .flatten()
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }

        usize::from(y)
            .checked_mul(usize::from(self.width))
            .and_then(|row| row.checked_add(usize::from(x)))
    }
}
