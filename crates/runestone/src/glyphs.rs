//! The Elder Futhark: the 24 runes, their names and sounds, and the text style that draws them
//! from the vendored Noto Sans Runic.

use rune::{Family, TextStyle};

/// One rune of the Elder Futhark.
pub struct Glyph {
    /// The character itself (U+16A0..U+16E8).
    pub rune: &'static str,
    pub name: &'static str,
    /// The Latin letter(s) it stands for.
    pub sound: &'static str,
    /// The aett (eighth) it belongs to: 0 Freyr's, 1 Hagal's, 2 Tyr's.
    pub aett: u8,
}

/// The 24 runes, in futhark order.
pub const FUTHARK: [Glyph; 24] = [
    Glyph {
        rune: "ᚠ",
        name: "Fehu",
        sound: "f",
        aett: 0,
    },
    Glyph {
        rune: "ᚢ",
        name: "Uruz",
        sound: "u",
        aett: 0,
    },
    Glyph {
        rune: "ᚦ",
        name: "Thurisaz",
        sound: "th",
        aett: 0,
    },
    Glyph {
        rune: "ᚨ",
        name: "Ansuz",
        sound: "a",
        aett: 0,
    },
    Glyph {
        rune: "ᚱ",
        name: "Raidho",
        sound: "r",
        aett: 0,
    },
    Glyph {
        rune: "ᚲ",
        name: "Kenaz",
        sound: "k",
        aett: 0,
    },
    Glyph {
        rune: "ᚷ",
        name: "Gebo",
        sound: "g",
        aett: 0,
    },
    Glyph {
        rune: "ᚹ",
        name: "Wunjo",
        sound: "w",
        aett: 0,
    },
    Glyph {
        rune: "ᚺ",
        name: "Hagalaz",
        sound: "h",
        aett: 1,
    },
    Glyph {
        rune: "ᚾ",
        name: "Naudhiz",
        sound: "n",
        aett: 1,
    },
    Glyph {
        rune: "ᛁ",
        name: "Isa",
        sound: "i",
        aett: 1,
    },
    Glyph {
        rune: "ᛃ",
        name: "Jera",
        sound: "j",
        aett: 1,
    },
    Glyph {
        rune: "ᛇ",
        name: "Eihwaz",
        sound: "ï",
        aett: 1,
    },
    Glyph {
        rune: "ᛈ",
        name: "Perthro",
        sound: "p",
        aett: 1,
    },
    Glyph {
        rune: "ᛉ",
        name: "Algiz",
        sound: "z",
        aett: 1,
    },
    Glyph {
        rune: "ᛊ",
        name: "Sowilo",
        sound: "s",
        aett: 1,
    },
    Glyph {
        rune: "ᛏ",
        name: "Tiwaz",
        sound: "t",
        aett: 2,
    },
    Glyph {
        rune: "ᛒ",
        name: "Berkanan",
        sound: "b",
        aett: 2,
    },
    Glyph {
        rune: "ᛖ",
        name: "Ehwaz",
        sound: "e",
        aett: 2,
    },
    Glyph {
        rune: "ᛗ",
        name: "Mannaz",
        sound: "m",
        aett: 2,
    },
    Glyph {
        rune: "ᛚ",
        name: "Laguz",
        sound: "l",
        aett: 2,
    },
    Glyph {
        rune: "ᛜ",
        name: "Ingwaz",
        sound: "ng",
        aett: 2,
    },
    Glyph {
        rune: "ᛞ",
        name: "Dagaz",
        sound: "d",
        aett: 2,
    },
    Glyph {
        rune: "ᛟ",
        name: "Othala",
        sound: "o",
        aett: 2,
    },
];

pub const COUNT: usize = FUTHARK.len();

/// The rune at `i`, wrapping around the futhark.
pub fn glyph(i: usize) -> &'static Glyph {
    &FUTHARK[i % COUNT]
}

/// A text style that draws runes: the vendored face, by its own family name.
pub fn style(size: f32, weight: f32) -> TextStyle {
    TextStyle::new(size, weight).family(Family::Named("Noto Sans Runic"))
}
