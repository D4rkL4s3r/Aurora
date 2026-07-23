use crate::actions::UiAction;
use crate::app::App;
use crate::pane::SortColumn;
use crate::ui::format::{entry_icon, format_date, format_size, type_label};

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::CentralPanel::default_margins().show(ui, |ui| {
        show_content(app, ui, actions);
    });
}

fn show_content(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let pane = &app.pane;

    if pane.is_loading() {
        ui.centered_and_justified(|ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Chargement…");
            });
        });
        return;
    }
    if let Some(error) = &pane.load_error {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(format!("⚠ Impossible d'ouvrir ce dossier : {error}"))
                    .color(ui.visuals().warn_fg_color),
            );
        });
        return;
    }

    let base = pane.displayed_base();
    let displayed_idx: Vec<usize> = match pane.instant_filter() {
        Some(query) => base
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect(),
        None => (0..base.len()).collect(),
    };
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
        .sense(egui::Sense::click())
        .column(egui_extras::Column::remainder().clip(true))
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .column(egui_extras::Column::auto())
        .header(22.0, |mut header| {
            header.col(|ui| {
                if ui.selectable_label(false, sort_label("Nom", SortColumn::Name)).clicked() {
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
            body.rows(22.0, displayed_idx.len(), |mut row| {
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
                let mut text =
                    egui::RichText::new(format!("{} {display_name}", entry_icon(entry)));
                if is_cut {
                    text = text.weak();
                }

                row.col(|ui| {
                    // Ellipse sur les noms trop longs plutôt que coupure brute
                    ui.add(egui::Label::new(text.clone()).truncate());
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
                        egui::Label::new(type_label(entry))
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(entry.modified.map(format_date).unwrap_or_default())
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                });

                let response = row.response();
                if response.clicked() {
                    let modifiers = response.ctx.input(|i| i.modifiers);
                    actions.push(UiAction::Select {
                        path: entry.path.clone(),
                        ctrl: modifiers.ctrl,
                        shift: modifiers.shift,
                    });
                }
                if response.double_clicked() {
                    if entry.is_dir {
                        actions.push(UiAction::Navigate(entry.path.clone()));
                    } else {
                        actions.push(UiAction::OpenFile(entry.path.clone()));
                    }
                }
                if response.secondary_clicked() {
                    actions.push(UiAction::ContextSelect(entry.path.clone()));
                }
                response.context_menu(|ui| {
                    if !entry.is_dir && ui.button("Ouvrir").clicked() {
                        actions.push(UiAction::OpenFile(entry.path.clone()));
                        ui.close();
                    }
                    if ui.button("Renommer (F2)").clicked() {
                        actions.push(UiAction::StartRename(entry.path.clone()));
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Copier (Ctrl+C)").clicked() {
                        actions.push(UiAction::CopySelection { cut: false });
                        ui.close();
                    }
                    if ui.button("Couper (Ctrl+X)").clicked() {
                        actions.push(UiAction::CopySelection { cut: true });
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Supprimer (Suppr)").clicked() {
                        actions.push(UiAction::AskDeleteSelection);
                        ui.close();
                    }
                });
            });
        });
}
