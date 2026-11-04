#[derive(Clone, Copy)]
pub struct Palette {
    pub background: u32,
    pub surface: u32,
    pub surface_hover: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub accent_text: u32,
    pub text: u32,
    pub muted: u32,
    pub quiet: u32,
    pub quiet_hover: u32,
    pub track: u32,
    pub progress: u32,
    pub danger: u32,
    pub danger_hover: u32,
}

pub const DARK: Palette = Palette {
    background: 0x0b1220,
    surface: 0x111a2e,
    surface_hover: 0x1b2740,
    accent: 0x2563eb,
    accent_hover: 0x3b82f6,
    accent_text: 0xf4f4f5,
    text: 0xf4f4f5,
    muted: 0x94a3b8,
    quiet: 0x1e293b,
    quiet_hover: 0x334155,
    track: 0x1e293b,
    progress: 0x22c55e,
    danger: 0xef4444,
    danger_hover: 0xf87171,
};

pub const NARROW_WIDTH: f32 = 720.;
