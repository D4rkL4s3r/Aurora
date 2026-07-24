use crate::actions::{Dialog, UiAction};
use crate::app::App;
use crate::pane::Pane;
use crate::ui::theme;
use std::path::PathBuf;

/// Breadcrumb cliquable : chaque segment du chemin navigue vers son dossier.
fn breadcrumb(pane: &Pane, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let mut ancestors: Vec<PathBuf> = pane
        .current_path
        .ancestors()
        .map(|p| p.to_path_buf())
        .collect();
    ancestors.reverse();

    let last = ancestors.len().saturating_sub(1);
    for (i, ancestor) in ancestors.iter().enumerate() {
        if i > 0 {
            ui.label(egui::RichText::new("›").weak());
        }
        let label = ancestor
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| ancestor.display().to_string());
        let text = if i == last {
            egui::RichText::new(label).strong()
        } else {
            egui::RichText::new(label)
        };
        if ui.selectable_label(false, text).clicked() && i != last {
            actions.push(UiAction::Navigate(ancestor.clone()));
        }
    }
}

/// Barre d'adresse éditable : Entrée valide le chemin, Échap ou perte de
/// focus annule et réaffiche le breadcrumb.
fn address_bar(app: &mut App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let tab = &mut app.tabs[app.active_tab];
    let pane = tab.pane_mut();
    let Some(text) = pane.address_edit.as_mut() else {
        return;
    };
    let width = (ui.available_width() - 340.0).max(160.0);
    let edit = ui.add(egui::TextEdit::singleline(text).desired_width(width));
    if pane.address_focus {
        edit.request_focus();
        pane.address_focus = false;
    }
    if edit.lost_focus() {
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let path = PathBuf::from(text.trim());
            if path.is_dir() {
                actions.push(UiAction::Navigate(path));
            } else {
                app.status = Some(format!("Dossier introuvable : {}", text.trim()));
            }
        }
        pane.address_edit = None;
    }
}

pub(crate) fn show(app: &mut App, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let frame = egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(10, 8));
    egui::Panel::top("toolbar").frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!app.pane().history.is_empty(), |ui| {
                if ui.button("⬅").on_hover_text("Précédent").clicked() {
                    app.go_back();
                }
            });
            if ui.button("⬆").on_hover_text("Dossier parent").clicked() {
                app.go_parent();
            }
            ui.separator();
            if app.pane().address_edit.is_some() {
                address_bar(app, ui, actions);
            } else {
                breadcrumb(app.pane(), ui, actions);
                let hint = app
                    .shortcuts
                    .menu_label("Saisir un chemin", crate::shortcuts::ShortcutAction::AddressBar);
                if ui.small_button("✏").on_hover_text(hint).clicked() {
                    app.pane_mut().open_address_bar();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_icon = if app.dark_theme { "☀" } else { "🌙" };
                if ui
                    .button(theme_icon)
                    .on_hover_text("Changer de thème")
                    .clicked()
                {
                    app.dark_theme = !app.dark_theme;
                    ui.ctx().set_theme(if app.dark_theme {
                        egui::ThemePreference::Dark
                    } else {
                        egui::ThemePreference::Light
                    });
                }
                if ui.button("⚙").on_hover_text("Paramètres").clicked() {
                    app.shortcuts_editor = Some(Default::default());
                }
                let split_hint = app
                    .shortcuts
                    .menu_label("Vue divisée", crate::shortcuts::ShortcutAction::ToggleSplit);
                if ui
                    .selectable_label(app.is_split(), "◫")
                    .on_hover_text(split_hint)
                    .clicked()
                {
                    app.toggle_split();
                }
                let filters_active = app.show_filters || app.pane().has_filters();
                if ui
                    .selectable_label(filters_active, "Filtres")
                    .on_hover_text("Filtres rapides (type, taille, date)")
                    .clicked()
                {
                    app.show_filters = !app.show_filters;
                    if !app.show_filters {
                        app.pane_mut().clear_filters();
                    }
                }
                let view_icon = if app.grid_view { "☰" } else { "⊞" };
                if ui
                    .button(view_icon)
                    .on_hover_text("Basculer liste / grille")
                    .clicked()
                {
                    app.grid_view = !app.grid_view;
                }
                let new_folder = egui::Button::new(
                    egui::RichText::new("+ Dossier").color(egui::Color32::WHITE),
                )
                .fill(theme::accent().gamma_multiply(0.55));
                if ui.add(new_folder).clicked() {
                    app.dialog = Some(Dialog::NewFolder {
                        name: String::new(),
                    });
                }
                if !app.pane().search_query.is_empty() && ui.button("✖").clicked() {
                    app.pane_mut().clear_search();
                    app.status = None;
                }
                let edit = ui
                    .add(
                        egui::TextEdit::singleline(&mut app.pane_mut().search_query)
                            .desired_width(220.0)
                            .hint_text("🔍 Rechercher…"),
                    )
                    .on_hover_text("Entrée : recherche aussi dans les sous-dossiers");
                if edit.changed() {
                    app.pane_mut().search_results = None;
                }
                if edit.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !app.pane().search_query.trim().is_empty()
                {
                    actions.push(UiAction::RunRecursiveSearch);
                }
            });
        });
    });
}
