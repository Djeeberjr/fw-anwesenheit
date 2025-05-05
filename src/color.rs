#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn to_grb(self) -> [u8; 3] {
        [self.g, self.r, self.b]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NamedColor {
    Red,
    Green,
    Blue,
    White,
    Off,
    Yellow,
    Cyan,
    Magenta,
}

impl From<NamedColor> for Color {
    fn from(named: NamedColor) -> Self {
        match named {
            NamedColor::Red => Color { r: 255, g: 0, b: 0 },
            NamedColor::Green => Color { r: 0, g: 255, b: 0 },
            NamedColor::Blue => Color { r: 0, g: 0, b: 255 },
            NamedColor::White => Color {
                r: 255,
                g: 255,
                b: 255,
            },
            NamedColor::Off => Color { r: 0, g: 0, b: 0 },
            NamedColor::Yellow => Color {
                r: 255,
                g: 255,
                b: 0,
            },
            NamedColor::Cyan => Color {
                r: 0,
                g: 255,
                b: 255,
            },
            NamedColor::Magenta => Color {
                r: 255,
                g: 0,
                b: 255,
            },
        }
    }
}
