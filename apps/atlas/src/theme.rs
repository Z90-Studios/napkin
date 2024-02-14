use egui::{epaint, style, Color32};
use serde::{Deserialize, Serialize};

/// Apply the given theme to a [`Context`](egui::Context).
pub fn set_theme(ctx: &egui::Context, theme: Theme) {
    let old = ctx.style().visuals.clone();
    ctx.set_visuals(theme.visuals(old));
}

/// Apply the given theme to a [`Style`](egui::Style).
///
/// # Example
///
/// ```rust
/// # use egui::__run_test_ctx;
/// # __run_test_ctx(|ctx| {
/// let mut style = (*ctx.style()).clone();
/// catppuccin_egui::set_style_theme(&mut style, catppuccin_egui::MACCHIATO);
/// ctx.set_style(style);
/// # });
/// ```
pub fn _set_style_theme(style: &mut egui::Style, theme: Theme) {
    let old = style.visuals.clone();
    style.visuals = theme.visuals(old);
}

fn make_widget_visual(
    old: style::WidgetVisuals,
    theme: &Theme,
    bg_fill: egui::Color32,
) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill,
        weak_bg_fill: bg_fill,
        bg_stroke: egui::Stroke {
            color: theme.overlay1,
            ..old.bg_stroke
        },
        fg_stroke: egui::Stroke {
            color: theme.text,
            ..old.fg_stroke
        },
        ..old
    }
}

/// The colors for a theme variant.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub rosewater: Color32,
    pub flamingo: Color32,
    pub pink: Color32,
    pub mauve: Color32,
    pub red: Color32,
    pub maroon: Color32,
    pub peach: Color32,
    pub yellow: Color32,
    pub green: Color32,
    pub teal: Color32,
    pub sky: Color32,
    pub sapphire: Color32,
    pub blue: Color32,
    pub lavender: Color32,
    pub text: Color32,
    pub subtext1: Color32,
    pub subtext0: Color32,
    pub overlay2: Color32,
    pub overlay1: Color32,
    pub overlay0: Color32,
    pub surface2: Color32,
    pub surface1: Color32,
    pub surface0: Color32,
    pub base: Color32,
    pub mantle: Color32,
    pub crust: Color32,
}

impl Theme {
    fn visuals(&self, old: egui::Visuals) -> egui::Visuals {
        let is_light = *self == LATTE;
        egui::Visuals {
            override_text_color: Some(self.text),
            hyperlink_color: self.rosewater,
            faint_bg_color: self.surface0,
            extreme_bg_color: self.crust,
            code_bg_color: self.mantle,
            warn_fg_color: self.peach,
            error_fg_color: self.maroon,
            window_fill: self.base,
            panel_fill: self.base,
            window_stroke: egui::Stroke {
                color: self.overlay1,
                ..old.window_stroke
            },
            widgets: style::Widgets {
                noninteractive: make_widget_visual(old.widgets.noninteractive, self, self.base),
                inactive: make_widget_visual(old.widgets.inactive, self, self.surface0),
                hovered: make_widget_visual(old.widgets.hovered, self, self.surface2),
                active: make_widget_visual(old.widgets.active, self, self.surface1),
                open: make_widget_visual(old.widgets.open, self, self.surface0),
            },
            selection: style::Selection {
                bg_fill: self.blue.linear_multiply(if is_light { 0.4 } else { 0.2 }),
                stroke: egui::Stroke {
                    color: self.overlay1,
                    ..old.selection.stroke
                },
            },
            window_shadow: epaint::Shadow {
                color: self.base,
                extrusion: 4.0,
                ..old.window_shadow
            },
            popup_shadow: epaint::Shadow {
                color: self.base,
                ..old.popup_shadow
            },
            dark_mode: !is_light,
            ..old
        }
    }
}


#[derive(PartialEq, Serialize, Deserialize)]
pub enum ColorScheme {
    Light {
        theme: Theme,

    },
    Dark {
        theme: Theme,
    },
}

impl ColorScheme {
    pub fn theme(&self) -> &Theme {
        match self {
            ColorScheme::Light { theme } => theme,
            ColorScheme::Dark { theme } => theme,
        }
    }
}

pub const LATTE: Theme = Theme {
    rosewater: Color32::from_rgb(220, 138, 120),
    flamingo: Color32::from_rgb(221, 120, 120),
    pink: Color32::from_rgb(234, 118, 203),
    mauve: Color32::from_rgb(136, 57, 239),
    red: Color32::from_rgb(210, 15, 57),
    maroon: Color32::from_rgb(230, 69, 83),
    peach: Color32::from_rgb(254, 100, 11),
    yellow: Color32::from_rgb(223, 142, 29),
    green: Color32::from_rgb(64, 160, 43),
    teal: Color32::from_rgb(23, 146, 153),
    sky: Color32::from_rgb(4, 165, 229),
    sapphire: Color32::from_rgb(32, 159, 181),
    blue: Color32::from_rgb(30, 102, 245),
    lavender: Color32::from_rgb(114, 135, 253),
    text: Color32::from_rgb(76, 79, 105),
    subtext1: Color32::from_rgb(92, 95, 119),
    subtext0: Color32::from_rgb(108, 111, 133),
    overlay2: Color32::from_rgb(124, 127, 147),
    overlay1: Color32::from_rgb(140, 143, 161),
    overlay0: Color32::from_rgb(156, 160, 176),
    surface2: Color32::from_rgb(172, 176, 190),
    surface1: Color32::from_rgb(188, 192, 204),
    surface0: Color32::from_rgb(204, 208, 218),
    base: Color32::from_rgb(239, 241, 245),
    mantle: Color32::from_rgb(230, 233, 239),
    crust: Color32::from_rgb(220, 224, 232),
};

pub const MACCHIATO: Theme = Theme {
    rosewater: Color32::from_rgb(244, 219, 214), // 244, 219, 214
    flamingo: Color32::from_rgb(240, 198, 198), // 240, 198, 198
    pink: Color32::from_rgb(245, 189, 230), // 245, 189, 230
    mauve: Color32::from_rgb(198, 160, 246), // 198, 160, 246
    red: Color32::from_rgb(237, 135, 150), // 237, 135, 150
    maroon: Color32::from_rgb(238, 153, 160), // 238, 153, 160
    peach: Color32::from_rgb(245, 169, 127), // 245, 169, 127
    yellow: Color32::from_rgb(238, 212, 159), // 238, 212, 159
    green: Color32::from_rgb(166, 218, 149), // 166, 218, 149
    teal: Color32::from_rgb(139, 213, 202), // 139, 213, 202
    sky: Color32::from_rgb(145, 215, 227), // 145, 215, 227
    sapphire: Color32::from_rgb(125, 196, 228), // 125, 196, 228
    blue: Color32::from_rgb(138, 173, 244), // 138, 173, 244
    lavender: Color32::from_rgb(183, 189, 248), // 183, 189, 248
    text: Color32::from_rgb(202, 211, 245), // 202, 211, 245
    subtext1: Color32::from_rgb(184, 192, 224), // 184, 192, 224
    subtext0: Color32::from_rgb(165, 173, 203), // 165, 173, 203
    overlay2: Color32::from_rgb(147, 154, 183), // 147, 154, 183
    overlay1: Color32::from_rgb(128, 135, 162), // 128, 135, 162
    overlay0: Color32::from_rgb(110, 115, 141), // 110, 115, 141
    surface2: Color32::from_rgb(91, 96, 120), // 91, 96, 120
    surface1: Color32::from_rgb(73, 77, 100), // 73, 77, 100
    surface0: Color32::from_rgb(54, 58, 79), // 54, 58, 79
    base: Color32::from_rgb(36, 39, 58), // 36, 39, 58
    mantle: Color32::from_rgb(30, 32, 48), // 30, 32, 48
    crust: Color32::from_rgb(24, 25, 38), // 24, 25, 38
};
