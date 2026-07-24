//! Écran d'accueil d'un volet sans dossier : colonne centrée façon page web,
//! cartes « accès rapide » à pastille colorée et cartes « lecteurs » avec
//! jauge d'espace disque.

use crate::actions::UiAction;
use crate::app::App;
use crate::ui::format::{format_size, quick_access_icon};
use crate::ui::theme;

const CARD_RADIUS: f32 = 12.0;

fn quick_access_color(name: &str) -> egui::Color32 {
    use egui::Color32;
    match name {
        "Bureau" => Color32::from_rgb(0x3a, 0x6e, 0xd8),
        "Documents" => Color32::from_rgb(0xd8, 0xa8, 0x3a),
        "Téléchargements" => Color32::from_rgb(0x2f, 0x9e, 0x6b),
        "Images" => Color32::from_rgb(0x8a, 0x4f, 0xd3),
        "Musique" => Color32::from_rgb(0xd3, 0x4f, 0x8a),
        "Vidéos" => Color32::from_rgb(0xd8, 0x5f, 0x3a),
        _ => Color32::from_rgb(0x6b, 0x72, 0x80),
    }
}

/// Carte cliquable : pastille d'icône colorée, titre, et pour les lecteurs
/// un sous-titre + une jauge d'utilisation.
fn card(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    icon: &str,
    icon_color: egui::Color32,
    title: &str,
    subtitle: Option<&str>,
    usage: Option<f32>,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();
        let bg = if response.hovered() {
            visuals.widgets.hovered.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };
        let painter = ui.painter();
        painter.rect_filled(rect, CARD_RADIUS, bg);

        let tile = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 33.0, rect.center().y),
            egui::Vec2::splat(36.0),
        );
        painter.rect_filled(tile, 10.0, icon_color);
        painter.text(
            tile.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(17.0),
            egui::Color32::WHITE,
        );

        let text_x = rect.left() + 62.0;
        let detailed = subtitle.is_some() || usage.is_some();
        if detailed {
            painter.text(
                egui::pos2(text_x, rect.top() + 13.0),
                egui::Align2::LEFT_TOP,
                title,
                egui::FontId::proportional(14.0),
                visuals.text_color(),
            );
            if let Some(subtitle) = subtitle {
                painter.text(
                    egui::pos2(text_x, rect.top() + 33.0),
                    egui::Align2::LEFT_TOP,
                    subtitle,
                    egui::FontId::proportional(11.5),
                    visuals.weak_text_color(),
                );
            }
            if let Some(fraction) = usage {
                let track = egui::Rect::from_min_max(
                    egui::pos2(text_x, rect.bottom() - 17.0),
                    egui::pos2(rect.right() - 16.0, rect.bottom() - 12.0),
                );
                painter.rect_filled(track, 2.5, visuals.extreme_bg_color);
                let fill_color = if fraction > 0.9 {
                    egui::Color32::from_rgb(0xd8, 0x5f, 0x3a)
                } else {
                    theme::accent()
                };
                let mut fill = track;
                fill.set_right(track.left() + track.width() * fraction.clamp(0.0, 1.0));
                painter.rect_filled(fill, 2.5, fill_color);
            }
        } else {
            painter.text(
                egui::pos2(text_x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(14.0),
                visuals.text_color(),
            );
        }
    }
    response
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).small().weak());
    ui.add_space(8.0);
}

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let column = 656.0_f32.min((ui.available_width() - 24.0).max(280.0));
            let margin = ((ui.available_width() - column) / 2.0).max(12.0);
            ui.add_space(40.0);
            ui.horizontal(|ui| {
                ui.add_space(margin);
                ui.vertical(|ui| {
                    ui.set_width(column);

                    ui.label(egui::RichText::new("Accueil").size(24.0).strong());
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Choisissez un emplacement pour commencer").weak(),
                    );
                    ui.add_space(26.0);

                    section(ui, "ACCÈS RAPIDE");
                    let quick_card = egui::Vec2::new((column - 24.0) / 3.0, 58.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 12.0);
                        for quick_access in &app.quick_access {
                            let icon = quick_access_icon(quick_access.name);
                            let color = quick_access_color(quick_access.name);
                            if card(ui, quick_card, icon, color, quick_access.name, None, None)
                                .clicked()
                            {
                                actions.push(UiAction::Navigate(quick_access.path.clone()));
                            }
                        }
                    });

                    ui.add_space(26.0);
                    section(ui, "LECTEURS");
                    let drive_card = egui::Vec2::new((column - 12.0) / 2.0, 74.0);
                    let drive_color = egui::Color32::from_rgb(0x4a, 0x54, 0x66);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 12.0);
                        for drive in &app.drives {
                            let title = drive.display().to_string();
                            let usage = app.drive_usage.get(drive);
                            let subtitle = usage.map(|(total, free)| {
                                format!(
                                    "{} libres sur {}",
                                    format_size(*free),
                                    format_size(*total)
                                )
                            });
                            let fraction = usage
                                .map(|(total, free)| 1.0 - *free as f32 / (*total).max(1) as f32);
                            if card(
                                ui,
                                drive_card,
                                "💾",
                                drive_color,
                                &title,
                                subtitle.as_deref(),
                                fraction,
                            )
                            .clicked()
                            {
                                actions.push(UiAction::Navigate(drive.clone()));
                            }
                        }
                    });
                    ui.add_space(32.0);
                });
            });
        });
}
