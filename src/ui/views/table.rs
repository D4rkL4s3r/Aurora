//! Vue liste détaillée : table virtualisée à colonnes triables.

use super::interactions::entry_interactions;
use crate::actions::UiAction;
use crate::app::App;
use crate::fs_ops::FileEntry;
use crate::pane::{Pane, SortColumn};
use crate::ui::format::{entry_icon, format_date, format_size, type_label};

pub(super) fn show(
    app: &App,
    pane: &Pane,
    ui: &mut egui::Ui,
    base: &[FileEntry],
    displayed_idx: &[usize],
    actions: &mut Vec<UiAction>,
) {
    let is_search = pane.search_results.is_some();
    let sort_label = |label: &str, column: SortColumn| -> String {
        if pane.sort_column == column {
            format!("{label} {}", if pane.sort_ascending { "⬆" } else { "⬇" })
        } else {
            label.to_owned()
        }
    };

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click_and_drag())
        .column(egui_extras::Column::remainder().clip(true))
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .header(26.0, |mut header| {
            header.col(|ui| {
                if ui
                    .selectable_label(false, sort_label("Nom", SortColumn::Name))
                    .clicked()
                {
                    actions.push(UiAction::SortBy(SortColumn::Name));
                }
            });
            header.col(|ui| {
                if ui
                    .selectable_label(false, sort_label("Taille", SortColumn::Size))
                    .clicked()
                {
                    actions.push(UiAction::SortBy(SortColumn::Size));
                }
            });
            header.col(|ui| {
                if ui
                    .selectable_label(false, sort_label("Type", SortColumn::Type))
                    .clicked()
                {
                    actions.push(UiAction::SortBy(SortColumn::Type));
                }
            });
            header.col(|ui| {
                if ui
                    .selectable_label(false, sort_label("Modifié", SortColumn::Modified))
                    .clicked()
                {
                    actions.push(UiAction::SortBy(SortColumn::Modified));
                }
            });
        })
        .body(|body| {
            body.rows(30.0, displayed_idx.len(), |mut row| {
                let entry = &base[displayed_idx[row.index()]];
                let is_selected = pane.selection.contains(&entry.path);
                row.set_selected(is_selected);

                let display_name = if is_search {
                    entry
                        .path
                        .strip_prefix(&pane.current_path)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| entry.name.clone())
                } else {
                    entry.name.clone()
                };
                let is_cut = app.clipboard_cut && app.clipboard.contains(&entry.path);

                row.col(|ui| {
                    let icon_tex = app.icons.borrow_mut().get(ui.ctx(), entry);
                    let mut name = egui::RichText::new(&display_name);
                    if is_cut {
                        name = name.weak();
                    }
                    ui.horizontal(|ui| {
                        match icon_tex {
                            Some(tex) => {
                                ui.add(
                                    egui::Image::new(&tex)
                                        .fit_to_exact_size(egui::Vec2::splat(16.0)),
                                );
                            }
                            None => {
                                ui.label(entry_icon(entry));
                            }
                        }
                        // Ellipse sur les noms trop longs plutôt que coupure brute
                        ui.add(egui::Label::new(name).truncate());
                    });
                });
                row.col(|ui| {
                    if !entry.is_dir {
                        ui.add(
                            egui::Label::new(format_size(entry.size))
                                .wrap_mode(egui::TextWrapMode::Extend),
                        );
                    }
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(type_label(entry)).wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(entry.modified.map(format_date).unwrap_or_default())
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });

                let response = row.response();
                entry_interactions(app, pane, entry, &response, actions);
            });
        });
}
