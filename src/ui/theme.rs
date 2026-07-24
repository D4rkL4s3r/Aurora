use egui::{Color32, CornerRadius, Stroke, Theme, Vec2};
use std::sync::atomic::{AtomicU32, Ordering};

/// Turquoise « aurore boréale », la couleur d'accent par défaut.
pub(crate) const DEFAULT_ACCENT: Color32 = Color32::from_rgb(0x35, 0xd6, 0xa6);

/// Accent courant, stocké en RGB pour être lisible depuis tout le rendu
/// sans traîner une référence à l'état de l'application.
static ACCENT_RGB: AtomicU32 = AtomicU32::new(0x35d6a6);

pub(crate) fn accent() -> Color32 {
    let rgb = ACCENT_RGB.load(Ordering::Relaxed);
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Change l'accent, réapplique le style et sauvegarde le choix.
pub(crate) fn set_accent(ctx: &egui::Context, color: Color32) {
    ACCENT_RGB.store(
        ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | color.b() as u32,
        Ordering::Relaxed,
    );
    apply(ctx);
    save_accent(color);
}

fn config_path() -> Option<std::path::PathBuf> {
    crate::fs_ops::config_dir().map(|dir| dir.join("theme.conf"))
}

/// Charge l'accent sauvegardé (`accent=RRGGBB`). À appeler avant [`apply`].
pub(crate) fn load_accent() {
    let Some(path) = config_path() else { return };
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("accent=") {
            if let Ok(rgb) = u32::from_str_radix(value.trim(), 16) {
                ACCENT_RGB.store(rgb & 0xff_ff_ff, Ordering::Relaxed);
            }
        }
    }
}

fn save_accent(color: Color32) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(
        path,
        format!("accent={:02x}{:02x}{:02x}\n", color.r(), color.g(), color.b()),
    );
}

/// Mélange linéaire de deux couleurs (t = 1 → entièrement `b`).
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
}

/// Applique le style Aurora aux deux variantes de thème.
/// La bascule sombre/clair (ThemePreference) conserve ce style.
pub(crate) fn apply(ctx: &egui::Context) {
    ctx.set_visuals_of(Theme::Dark, dark_visuals());
    ctx.set_visuals_of(Theme::Light, light_visuals());
    for theme in [Theme::Dark, Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            style.spacing.item_spacing = Vec2::new(8.0, 6.0);
            style.spacing.button_padding = Vec2::new(10.0, 5.0);
        });
    }
}

fn rounded(mut visuals: egui::Visuals) -> egui::Visuals {
    let radius = CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = radius;
    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.active.corner_radius = radius;
    visuals.widgets.open.corner_radius = radius;
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.hyperlink_color = accent();
    visuals
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = rounded(egui::Visuals::dark());
    visuals.panel_fill = Color32::from_rgb(16, 17, 21);
    visuals.extreme_bg_color = Color32::from_rgb(10, 11, 14);
    visuals.faint_bg_color = Color32::from_rgb(27, 29, 35);
    visuals.selection.bg_fill = mix(accent(), Color32::BLACK, 0.62);
    visuals.selection.stroke = Stroke::new(1.0, accent());
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = rounded(egui::Visuals::light());
    visuals.panel_fill = Color32::from_rgb(248, 249, 251);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = Color32::from_rgb(240, 242, 246);
    visuals.selection.bg_fill = mix(accent(), Color32::WHITE, 0.78);
    visuals.selection.stroke = Stroke::new(1.0, mix(accent(), Color32::BLACK, 0.25));
    visuals
}
