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
            style.spacing.item_spacing = Vec2::new(10.0, 8.0);
            style.spacing.button_padding = Vec2::new(12.0, 6.0);
            style.spacing.menu_margin = egui::Margin::same(8);
            style.spacing.window_margin = egui::Margin::same(14);
            // Scrollbars fines en surimpression, apparaissant au survol.
            style.spacing.scroll = egui::style::ScrollStyle::floating();
            // Transitions de survol un peu plus lentes = moins abruptes.
            style.animation_time = 0.15;
        });
    }
}

/// Adoucissements communs : coins ronds, boutons sans bordure, ombres
/// diffuses, curseur main sur les éléments cliquables.
fn soften(mut visuals: egui::Visuals) -> egui::Visuals {
    let radius = CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = radius;
    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.active.corner_radius = radius;
    visuals.widgets.open.corner_radius = radius;
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(10);
    // Pas de bordure ni de grossissement au survol : seul le fond change.
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.expansion = 0.0;
    visuals.widgets.active.expansion = 0.0;
    visuals.window_shadow = egui::Shadow {
        offset: [0, 8],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(70),
    };
    visuals.popup_shadow = egui::Shadow {
        offset: [0, 4],
        blur: 14,
        spread: 0,
        color: Color32::from_black_alpha(50),
    };
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    visuals.hyperlink_color = accent();
    visuals
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = soften(egui::Visuals::dark());
    visuals.panel_fill = Color32::from_rgb(17, 18, 23);
    visuals.window_fill = Color32::from_rgb(23, 24, 30);
    visuals.extreme_bg_color = Color32::from_rgb(11, 12, 16);
    visuals.faint_bg_color = Color32::from_rgb(28, 30, 37);
    // Séparateurs à peine visibles plutôt que lignes franches.
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_white_alpha(12));
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(31, 33, 41);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(31, 33, 41);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(40, 43, 53);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 43, 53);
    visuals.widgets.active.weak_bg_fill = mix(accent(), Color32::from_rgb(31, 33, 41), 0.75);
    visuals.widgets.active.bg_fill = visuals.widgets.active.weak_bg_fill;
    visuals.selection.bg_fill = mix(accent(), Color32::BLACK, 0.62);
    visuals.selection.stroke = Stroke::new(1.0, accent());
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = soften(egui::Visuals::light());
    visuals.panel_fill = Color32::from_rgb(250, 250, 252);
    visuals.window_fill = Color32::WHITE;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = Color32::from_rgb(242, 243, 247);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_black_alpha(16));
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(236, 238, 243);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(236, 238, 243);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(227, 230, 237);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(227, 230, 237);
    visuals.widgets.active.weak_bg_fill = mix(accent(), Color32::WHITE, 0.72);
    visuals.widgets.active.bg_fill = visuals.widgets.active.weak_bg_fill;
    visuals.selection.bg_fill = mix(accent(), Color32::WHITE, 0.78);
    visuals.selection.stroke = Stroke::new(1.0, mix(accent(), Color32::BLACK, 0.25));
    visuals
}
