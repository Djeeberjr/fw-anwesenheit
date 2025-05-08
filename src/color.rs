use rgb::Rgb;

#[derive(Debug)]
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

impl Into<Rgb<u8>> for NamedColor {
    fn into(self) -> Rgb<u8> {
        match self {
            NamedColor::Red => Rgb { r: 150, g: 0, b: 0 },
            NamedColor::Green => Rgb { r: 0, g: 150, b: 0 },
            NamedColor::Blue => Rgb { r: 0, g: 0, b: 150 },
            NamedColor::White => Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            NamedColor::Off => Rgb { r: 0, g: 0, b: 0 },
            NamedColor::Yellow => Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
            NamedColor::Cyan => Rgb {
                r: 0,
                g: 255,
                b: 255,
            },
            NamedColor::Magenta => Rgb {
                r: 255,
                g: 0,
                b: 255,
            },
        }
    }
}


impl IntoIterator for NamedColor {
    type Item = Self;

    type IntoIter = std::vec::IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

