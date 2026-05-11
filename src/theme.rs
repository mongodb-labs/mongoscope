use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self { Theme::Dark => Theme::Light, Theme::Light => Theme::Dark }
    }
    pub fn label(self) -> &'static str {
        match self { Theme::Dark => "Dark", Theme::Light => "Light" }
    }
    pub fn palette(self) -> Palette {
        match self { Theme::Dark => Palette::dark(), Theme::Light => Palette::light() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    #[default]
    Compact,
    Comfy,
}

impl Density {
    pub fn toggle(self) -> Self {
        match self { Density::Compact => Density::Comfy, Density::Comfy => Density::Compact }
    }
    pub fn label(self) -> &'static str {
        match self { Density::Compact => "Compact", Density::Comfy => "Comfy" }
    }
    pub fn row_height(self) -> f32 {
        match self { Density::Compact => 28.0, Density::Comfy => 34.0 }
    }
    pub fn header_height(self) -> f32 {
        match self { Density::Compact => 28.0, Density::Comfy => 34.0 }
    }
    pub fn tab_height(self) -> f32 {
        match self { Density::Compact => 30.0, Density::Comfy => 36.0 }
    }
    pub fn fs_base(self) -> f32 {
        match self { Density::Compact => 12.0, Density::Comfy => 13.0 }
    }
    pub fn fs_small(self) -> f32 {
        match self { Density::Compact => 11.0, Density::Comfy => 12.0 }
    }
    pub fn fs_mono(self) -> f32 {
        match self { Density::Compact => 11.5, Density::Comfy => 12.5 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dock {
    #[default]
    Bottom,
    Right,
}

/// All design-token colors. Values converted from oklch in mongoscope.css.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub bg1: Color,
    pub bg2: Color,
    pub bg_sel: Color,
    pub bg_hover: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub fg_dim2: Color,
    pub border: Color,
    pub border2: Color,
    pub accent: Color,
    pub accent_fg: Color,
    pub warn: Color,
    pub danger: Color,
    pub ok: Color,
    pub op_read: Color,
    pub op_write: Color,
    pub op_agg: Color,
    pub op_delete: Color,
    pub t_parse: Color,
    pub t_auth: Color,
    pub t_plan: Color,
    pub t_exec: Color,
    pub t_ser: Color,
    pub t_net: Color,
    pub tok_key: Color,
    pub tok_str: Color,
    pub tok_num: Color,
    pub tok_lit: Color,
    pub tok_call: Color,
    pub tok_br: Color,
    pub tok_p: Color,
    pub tok_colon: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            bg:       frgb(0.0445, 0.0529, 0.0619),
            bg1:      frgb(0.0728, 0.0831, 0.0941),
            bg2:      frgb(0.1025, 0.1147, 0.1278),
            bg_sel:   frgba(0.1260, 0.2268, 0.1414, 0.55),
            bg_hover: frgb(0.0935, 0.1056, 0.1186),
            fg:       frgb(0.9150, 0.9229, 0.9289),
            fg_dim:   frgb(0.5291, 0.5532, 0.5713),
            fg_dim2:  frgb(0.3698, 0.3926, 0.4096),
            border:   frgb(0.1300, 0.1426, 0.1562),
            border2:  frgb(0.1680, 0.1811, 0.1951),
            accent:   frgb(0.4267, 0.8204, 0.4972),
            accent_fg:frgb(0.0370, 0.0545, 0.0673),
            warn:     frgb(0.9668, 0.7211, 0.2406),
            danger:   frgb(0.9817, 0.4094, 0.3878),
            ok:       frgb(0.4267, 0.8204, 0.4972),
            op_read:  frgb(0.1678, 0.7417, 0.9606),
            op_write: frgb(0.9542, 0.7257, 0.2986),
            op_agg:   frgb(0.7455, 0.5807, 1.0000),
            op_delete:frgb(1.0000, 0.4350, 0.4113),
            t_parse:  frgb(0.2274, 0.3500, 0.4500),
            t_auth:   frgb(0.6000, 0.3800, 0.2800),
            t_plan:   frgb(0.7455, 0.5807, 1.0000),
            t_exec:   frgb(0.4267, 0.8204, 0.4972),
            t_ser:    frgb(0.9542, 0.7257, 0.2986),
            t_net:    frgb(0.1678, 0.7417, 0.9606),
            tok_key:  frgb(0.4766, 0.7403, 1.0000),
            tok_str:  frgb(1.0000, 0.6144, 0.4649),
            tok_num:  frgb(0.7147, 0.6661, 0.9995),
            tok_lit:  frgb(1.0000, 0.5048, 0.4753),
            tok_call: frgb(0.2274, 0.8084, 0.8373),
            tok_br:   frgb(0.6002, 0.6249, 0.6434),
            tok_p:    frgb(0.4255, 0.4488, 0.4662),
            tok_colon:frgb(0.4255, 0.4488, 0.4662),
        }
    }

    pub fn light() -> Self {
        Self {
            bg:       hex(0xfafaf8),
            bg1:      hex(0xf2f2ee),
            bg2:      hex(0xe8e8e2),
            bg_sel:   rgba(0x50, 0xb8, 0x80, 0.55),
            bg_hover: hex(0xededea),
            fg:       hex(0x262830),
            fg_dim:   hex(0x6a707c),
            fg_dim2:  hex(0x9098a8),
            border:   hex(0xdedede),
            border2:  hex(0xcccccc),
            accent:   hex(0x2e8a58),
            accent_fg:hex(0xfafafa),
            warn:     hex(0x9a7020),
            danger:   hex(0xb83030),
            ok:       hex(0x2e8a58),
            op_read:  hex(0x2870b0),
            op_write: hex(0x8a6010),
            op_agg:   hex(0x7030a0),
            op_delete:hex(0xb03030),
            t_parse:  hex(0xb0bcc8),
            t_auth:   hex(0xc8a888),
            t_plan:   hex(0xb888d8),
            t_exec:   hex(0x80c8a0),
            t_ser:    hex(0xc8b060),
            t_net:    hex(0x90b8d0),
            tok_key:  hex(0x2850a0),
            tok_str:  hex(0x9a4a10),
            tok_num:  hex(0x6040b0),
            tok_lit:  hex(0xa03030),
            tok_call: hex(0x206080),
            tok_br:   hex(0x707888),
            tok_p:    hex(0x909898),
            tok_colon:hex(0x909898),
        }
    }

    pub fn with_alpha(c: Color, a: f32) -> Color {
        Color { r: c.r, g: c.g, b: c.b, a }
    }
}

fn hex(v: u32) -> Color {
    Color::from_rgb8(
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a }
}

fn frgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

fn frgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color { r, g, b, a }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_palette_accent_is_greenish() {
        let p = Palette::dark();
        assert!(p.accent.g > p.accent.r, "accent should be green-dominant");
    }

    #[test]
    fn theme_toggle_roundtrip() {
        assert_eq!(Theme::Dark.toggle().toggle(), Theme::Dark);
    }

    #[test]
    fn density_compact_smaller_than_comfy() {
        assert!(Density::Compact.row_height() < Density::Comfy.row_height());
        assert!(Density::Compact.fs_base() < Density::Comfy.fs_base());
    }
}
