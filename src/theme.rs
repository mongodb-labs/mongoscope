use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Dark => Palette::dark(),
            Theme::Light => Palette::light(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    #[default]
    Compact,
    Comfy,
}

impl Density {
    pub fn row_height(self) -> f32 {
        match self { Density::Compact => 26.0, Density::Comfy => 32.0 }
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
            bg:       hex(0x22252e),
            bg1:      hex(0x292d38),
            bg2:      hex(0x30353f),
            bg_sel:   rgba(0x3d, 0x7a, 0x5a, 0.55),
            bg_hover: hex(0x2d3140),
            fg:       hex(0xeef0f4),
            fg_dim:   hex(0x8d96a8),
            fg_dim2:  hex(0x5e6678),
            border:   hex(0x363b48),
            border2:  hex(0x3e4454),
            accent:   hex(0x5fc490),
            accent_fg:hex(0x1a1e26),
            warn:     hex(0xd4a843),
            danger:   hex(0xe06060),
            ok:       hex(0x5fc490),
            op_read:  hex(0x6fb3e8),
            op_write: hex(0xd4a843),
            op_agg:   hex(0xb07de8),
            op_delete:hex(0xe07070),
            t_parse:  hex(0x607080),
            t_auth:   hex(0xa07060),
            t_plan:   hex(0x9070c0),
            t_exec:   hex(0x5fb880),
            t_ser:    hex(0xb09040),
            t_net:    hex(0x6090b0),
            tok_key:  hex(0x88aadd),
            tok_str:  hex(0xd4916a),
            tok_num:  hex(0xb08ed4),
            tok_lit:  hex(0xd47070),
            tok_call: hex(0x6ab8cc),
            tok_br:   hex(0x9098a8),
            tok_p:    hex(0x606878),
            tok_colon:hex(0x606878),
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
