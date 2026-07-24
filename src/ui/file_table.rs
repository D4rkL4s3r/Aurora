use crate::actions::{DragPayload, UiAction};
use crate::app::App;
use crate::fs_ops::FileEntry;
use crate::pane::{Pane, SortColumn};
use crate::ui::format::{entry_icon, entry_tile_color, format_date, format_size, type_label};
use crate::ui::theme;

pub(crate) fn show(app: &App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    egui::CentralPanel::default_margins().show(ui, |ui| {
        // Chaque onglet garde son propre état d'affichage (scroll, largeurs).
        ui.push_id(("tab", app.active_tab), |ui| {
            if app.is_split() {
                ui.columns(app.tab().panes.len(), |cols| {
                    for (idx, col) in cols.iter_mut().enumerate() {
                        pane_frame(app, idx, col, actions);
                    }
                });
            } else {
                show_content(app, 0, ui, actions);
                background_drop(&app.tab().panes[0], ui, actions);
            }
        });
    });
}

/// Un volet de la vue divisée : cadre (accentué pour le volet actif),
/// clic n'importe où dedans → devient le volet actif.
fn pane_frame(app: &App, idx: usize, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let active = idx == app.tab().active_pane;
    let stroke = if active {
        egui::Stroke::new(1.5, theme::ACCENT.gamma_multiply(0.7))
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };
    egui::Frame::default()
        .stroke(stroke)
        .corner_radius(6.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            // Poussé avant les actions des entrées : le changement de volet actif
            // s'applique donc avant la sélection issue du même clic.
            if !active
                && ui.rect_contains_pointer(ui.max_rect())
                && ui.input(|i| i.pointer.any_pressed())
            {
                actions.push(UiAction::FocusPane(idx));
            }
            ui.push_id(idx, |ui| show_content(app, idx, ui, actions));
            background_drop(&app.tab().panes[idx], ui, actions);
        });
}

/// Cible de dépôt « fond du volet » : un glisser-déposer relâché sur le volet
/// (hors d'un dossier précis) atterrit dans son dossier courant.
fn background_drop(pane: &Pane, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let response = ui.response();
    if response.dnd_hover_payload::<DragPayload>().is_some() {
        ui.painter().rect_stroke(
            ui.max_rect(),
            6.0,
            egui::Stroke::new(1.5, theme::ACCENT.gamma_multiply(0.6)),
            egui::StrokeKind::Inside,
        );
    }
    if let Some(payload) = response.dnd_release_payload::<DragPayload>() {
        let copy = ui.input(|i| i.modifiers.ctrl);
        actions.push(UiAction::DropPaths {
            paths: payload.paths.clone(),
            dest: pane.current_path.clone(),
            copy,
        });
        egui::DragAndDrop::clear_payload(ui.ctx());
    }
}

fn elide(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        name.to_owned()
    } else {
        let mut short: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        short.push('…');
        short
    }
}

fn entry_context_menu(entry: &FileEntry, response: &egui::Response, actions: &mut Vec<UiAction>) {
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
}

/// Gère un glisser-déposer sur cette entrée : départ de drag (la sélection
/// suit l'entrée traînée) et, pour un dossier, cible de dépôt.
fn entry_drag_and_drop(
    pane: &Pane,
    entry: &FileEntry,
    response: &egui::Response,
    actions: &mut Vec<UiAction>,
) {
    if response.drag_started() {
        let paths = if pane.selection.contains(&entry.path) {
            pane.selected_in_order()
        } else {
            actions.push(UiAction::ContextSelect(entry.path.clone()));
            vec![entry.path.clone()]
        };
        egui::DragAndDrop::set_payload(&response.ctx, DragPayload { paths });
    }
    if !entry.is_dir {
        return;
    }
    let Some(payload) = response.dnd_hover_payload::<DragPayload>() else {
        return;
    };
    if payload.paths.contains(&entry.path) {
        return; // on ne dépose pas un dossier sur lui-même
    }
    let painter = response.ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("dnd_target"),
    ));
    painter.rect_stroke(
        response.rect,
        6.0,
        egui::Stroke::new(1.5, theme::ACCENT),
        egui::StrokeKind::Inside,
    );
    if response.dnd_release_payload::<DragPayload>().is_some() {
        let copy = response.ctx.input(|i| i.modifiers.ctrl);
        actions.push(UiAction::DropPaths {
            paths: payload.paths.clone(),
            dest: entry.path.clone(),
            copy,
        });
        egui::DragAndDrop::clear_payload(&response.ctx);
    }
}

fn entry_interactions(
    pane: &Pane,
    entry: &FileEntry,
    response: &egui::Response,
    actions: &mut Vec<UiAction>,
) {
    entry_drag_and_drop(pane, entry, response, actions);
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
    entry_context_menu(entry, response, actions);
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
            painter.rect_filled(rect, 8.0, visuals.selection.bg_fill.gamma_multiply(0.55));
            painter.rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.5, theme::ACCENT),
                egui::StrokeKind::Inside,
            );
        } else if response.hovered() {
            painter.rect_filled(rect, 8.0, visuals.faint_bg_color);
        }

        let tile = egui::Rect::from_center_size(
            egui::Pos2::new(rect.center().x, rect.top() + 36.0),
            egui::Vec2::splat(48.0),
        );
        let icon_tex = app.icons.borrow_mut().get(&response.ctx, entry);
        if let Some(tex) = icon_tex {
            // Vraie icône système sur un fond neutre discret.
            painter.rect_filled(tile, 10.0, visuals.faint_bg_color);
            let icon_rect = egui::Rect::from_center_size(tile.center(), egui::Vec2::splat(32.0));
            let tint = if is_cut {
                egui::Color32::from_white_alpha(110)
            } else {
                egui::Color32::WHITE
            };
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
            painter.rect_filled(tile, 10.0, tile_color);
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

    entry_interactions(pane, entry, &response, actions);
}

fn show_grid(
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
                ui.spacing_mut().item_spacing = egui::Vec2::new(10.0, 10.0);
                for &i in displayed_idx {
                    grid_card(app, pane, &base[i], ui, actions);
                }
            });
        });
}

fn show_content(app: &App, pane_idx: usize, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let pane = &app.tab().panes[pane_idx];

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

    if app.grid_view {
        show_grid(app, pane, ui, base, &displayed_idx, actions);
        return;
    }

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
            body.rows(26.0, displayed_idx.len(), |mut row| {
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
                entry_interactions(pane, entry, &response, actions);
            });
        });
}
