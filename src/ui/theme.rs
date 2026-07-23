use egui::{Color32, CornerRadius, Stroke, Theme, Vec2};

/// Turquoise « aurore boréale », la couleur d'accent de l'application.
pub(crate) const ACCENT: Color32 = Color32::from_rgb(0x35, 0xd6, 0xa6);

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
    visuals.hyperlink_color = ACCENT;
    visuals
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = rounded(egui::Visuals::dark());
    visuals.panel_fill = Color32::from_rgb(24, 26, 31);
    visuals.extreme_bg_color = Color32::from_rgb(15, 17, 21);
    visuals.faint_bg_color = Color32::from_rgb(31, 34, 40);
    visuals.selection.bg_fill = Color32::from_rgb(16, 84, 66);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = rounded(egui::Visuals::light());
    visuals.panel_fill = Color32::from_rgb(248, 249, 251);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = Color32::from_rgb(240, 242, 246);
    visuals.selection.bg_fill = Color32::from_rgb(205, 242, 228);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(0x1f, 0x9d, 0x78));
    visuals
}
