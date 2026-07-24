//! Vue grille : cartes avec miniature, icône système ou tuile colorée.

use super::interactions::entry_interactions;
use crate::actions::UiAction;
use crate::app::App;
use crate::fs_ops::FileEntry;
use crate::pane::Pane;
use crate::ui::format::{entry_icon, entry_tile_color, format_size};
use crate::ui::theme;

fn elide(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        name.to_owned()
    } else {
        let mut short: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        short.push('…');
        short
    }
}

fn grid_card(
    app: &App,
    pane: &Pane,
    entry: &FileEntry,
    ui: &mut egui::Ui,
    actions: &mut Vec<UiAction>,
) {
    const CARD: egui::Vec2 = egui::Vec2::new(104.0, 118.0);
    let (rect, response) = ui.allocate_exact_size(CARD, egui::Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();
        let selected = pane.selection.contains(&entry.path);
        let is_cut = app.clipboard_cut && app.clipboard.contains(&entry.path);
        let painter = ui.painter();

        if selected {
            painter.rect_filled(rect, 10.0, visuals.selection.bg_fill.gamma_multiply(0.55));
            painter.rect_stroke(
                rect,
                10.0,
                egui::Stroke::new(1.2, theme::accent().gamma_multiply(0.9)),
                egui::StrokeKind::Inside,
            );
        } else if response.hovered() {
            painter.rect_filled(rect, 10.0, visuals.faint_bg_color);
        }

        let tile = egui::Rect::from_center_size(
            egui::Pos2::new(rect.center().x, rect.top() + 36.0),
            egui::Vec2::splat(48.0),
        );
        let tint = if is_cut {
            egui::Color32::from_white_alpha(110)
        } else {
            egui::Color32::WHITE
        };
        if let Some(thumb) = app.thumbs.borrow_mut().get(entry) {
            // Miniature réelle, ajustée sans déformation dans la zone d'icône.
            let area = egui::Rect::from_center_size(tile.center(), egui::Vec2::new(88.0, 60.0));
            let size = thumb.size_vec2();
            let scale = (area.width() / size.x).min(area.height() / size.y);
            let draw_rect = egui::Rect::from_center_size(area.center(), size * scale);
            painter.image(
                thumb.id(),
                draw_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        } else if let Some(tex) = app.icons.borrow_mut().get(&response.ctx, entry) {
            // Vraie icône système sur un fond neutre discret.
            painter.rect_filled(tile, 12.0, visuals.faint_bg_color);
            let icon_rect = egui::Rect::from_center_size(tile.center(), egui::Vec2::splat(32.0));
            painter.image(
                tex.id(),
                icon_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        } else {
            let tile_color = if is_cut {
                entry_tile_color(entry).gamma_multiply(0.4)
            } else {
                entry_tile_color(entry)
            };
            painter.rect_filled(tile, 12.0, tile_color);
            painter.text(
                tile.center(),
                egui::Align2::CENTER_CENTER,
                entry_icon(entry),
                egui::FontId::proportional(24.0),
                egui::Color32::WHITE,
            );
        }

        let text_color = if is_cut {
            visuals.weak_text_color()
        } else {
            visuals.text_color()
        };
        painter.text(
            egui::Pos2::new(rect.center().x, rect.top() + 68.0),
            egui::Align2::CENTER_TOP,
            elide(&entry.name, 14),
            egui::FontId::proportional(12.5),
            text_color,
        );
        if !entry.is_dir {
            painter.text(
                egui::Pos2::new(rect.center().x, rect.top() + 88.0),
                egui::Align2::CENTER_TOP,
                format_size(entry.size),
                egui::FontId::proportional(10.5),
                visuals.weak_text_color(),
            );
        }
    }

    entry_interactions(app, pane, entry, &response, actions);
}

pub(super) fn show(
    app: &App,
    pane: &Pane,
    ui: &mut egui::Ui,
    base: &[FileEntry],
    displayed_idx: &[usize],
    actions: &mut Vec<UiAction>,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::new(12.0, 12.0);
                for &i in displayed_idx {
                    grid_card(app, pane, &base[i], ui, actions);
                }
            });
        });
}
