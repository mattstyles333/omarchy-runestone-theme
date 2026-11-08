//! The Runestone palette: one source of truth for every colour, in the scenes and in the
//! Omarchy `colors.toml` this repository ships.
//!
//! Every constant here is an ordinary sRGB hex — the same string that goes into `colors.toml`, so
//! the wallpaper on screen and the desktop's own chrome are the same colour. Rune paints in
//! linear light, so [`decode`] is the one conversion between them, and nothing else in the
//! repository needs to know about it.

use rune::Color;

/// sRGB (0..1 per channel) to linear light, the encoding Rune's `Color` is in.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A colour from a `0xRRGGBB` sRGB literal, in the linear light Rune paints in.
pub fn decode(hex: u32) -> Color {
    Color {
        r: srgb_to_linear(((hex >> 16) & 0xff) as f32 / 255.0),
        g: srgb_to_linear(((hex >> 8) & 0xff) as f32 / 255.0),
        b: srgb_to_linear((hex & 0xff) as f32 / 255.0),
        a: 1.0,
    }
}

/// Nothing at all: the last stop of every soft gradient.
pub const CLEAR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};
/// True black, for shading. The same value in sRGB and in linear light.
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// `a` and `b` mixed by `t` (`0.0..1.0`), alpha included: how a rune goes from coal to burning.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

// ---------------------------------------------------------------- the scene colours

/// The wallpaper's base: near-black with the least trace of blue in it.
pub const OBSIDIAN_HEX: u32 = 0x05060a;
/// The corners of the frame, below the base.
pub const OBSIDIAN_DEEP_HEX: u32 = 0x020304;

/// The standing stone's body, its lit face, and the edge that catches the light.
pub const STONE_HEX: u32 = 0x0e1116;
pub const STONE_LIT_HEX: u32 = 0x1e242e;
pub const STONE_EDGE_HEX: u32 = 0x46536a;

/// Ember: the warm light the runes are cut from, at its hottest, its pale core and its coal.
pub const EMBER_HEX: u32 = 0xff7a29;
pub const EMBER_GOLD_HEX: u32 = 0xffb347;
pub const EMBER_PALE_HEX: u32 = 0xffe3b8;
pub const EMBER_DEEP_HEX: u32 = 0x8c2f08;

/// Aurora: the cool light behind everything.
pub const AURORA_TEAL_HEX: u32 = 0x2fe3c4;
pub const AURORA_VIOLET_HEX: u32 = 0x7c5cff;
pub const AURORA_BLUE_HEX: u32 = 0x3fa9f5;

/// Warm parchment ink, for anything that has to sit on the obsidian base as text.
pub const FOREGROUND_HEX: u32 = 0xe9e3d7;

/// The accent Omarchy highlights with: the ember itself.
pub const ACCENT_HEX: u32 = 0xff8a3d;

pub fn obsidian() -> Color {
    decode(OBSIDIAN_HEX)
}
pub fn obsidian_deep() -> Color {
    decode(OBSIDIAN_DEEP_HEX)
}
pub fn stone() -> Color {
    decode(STONE_HEX)
}
pub fn stone_lit() -> Color {
    decode(STONE_LIT_HEX)
}
pub fn stone_edge() -> Color {
    decode(STONE_EDGE_HEX)
}
pub fn ember() -> Color {
    decode(EMBER_HEX)
}
pub fn ember_gold() -> Color {
    decode(EMBER_GOLD_HEX)
}
pub fn ember_pale() -> Color {
    decode(EMBER_PALE_HEX)
}
pub fn ember_deep() -> Color {
    decode(EMBER_DEEP_HEX)
}
pub fn aurora_teal() -> Color {
    decode(AURORA_TEAL_HEX)
}
pub fn aurora_violet() -> Color {
    decode(AURORA_VIOLET_HEX)
}
pub fn aurora_blue() -> Color {
    decode(AURORA_BLUE_HEX)
}

// ---------------------------------------------------------------- the Omarchy theme

/// The `colors.toml` that ships as this theme's palette, generated from the constants above so
/// the theme and the wallpapers can never drift apart. Same keys as the themes Omarchy ships.
pub fn colors_toml() -> String {
    let hex = |v: u32| format!("#{v:06x}");
    format!(
        r##"# Runestone. Generated from crates/runestone/src/palette.rs:
# `cargo run --release -p runestone-render -- --colors-toml > colors.toml`.

mode = "dark"

accent = "{accent}"
selection = "{selection}"
muted = "{muted}"

background = "{background}"
dark_background = "{dark_background}"
darker_background = "{darker_background}"
lighter_background = "{lighter_background}"

foreground = "{foreground}"
dark_foreground = "{dark_foreground}"
light_foreground = "{light_foreground}"
bright_foreground = "{bright_foreground}"

red = "#e05a3a"
yellow = "#ffc861"
orange = "{orange}"
green = "#6fd3a8"
cyan = "{cyan}"
blue = "#5aa9ff"
magenta = "#b98cff"
brown = "#8a6242"

bright_red = "#ff7455"
bright_yellow = "#ffd98a"
bright_green = "#9de8c2"
bright_cyan = "#6ff0dc"
bright_blue = "#8cc4ff"
bright_magenta = "#d3b0ff"
"##,
        accent = hex(ACCENT_HEX),
        selection = "#181d26",
        muted = "#6d7480",
        background = hex(OBSIDIAN_HEX),
        dark_background = hex(OBSIDIAN_DEEP_HEX),
        darker_background = "#010102",
        lighter_background = "#12151c",
        foreground = hex(FOREGROUND_HEX),
        dark_foreground = "#8b8578",
        light_foreground = "#f4efe6",
        bright_foreground = "#fffaf2",
        orange = hex(EMBER_HEX),
        cyan = hex(AURORA_TEAL_HEX),
    )
}
