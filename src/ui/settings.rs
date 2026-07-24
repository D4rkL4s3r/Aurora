//! Fenêtre de paramètres (bouton ⚙) : couleur d'accent et raccourcis
//! clavier (cliquer sur un raccourci puis appuyer sur la nouvelle combinaison).

use crate::app::App;
use crate::shortcuts::{ShortcutAction, ShortcutsEditor};
use crate::ui::theme;

pub(crate) fn show(app: &mut App, ctx: &egui::Context) {
    if app.shortcuts_editor.is_none() {
        return;
    }
    let mut open = true;
    egui::Window::new("Paramètres")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Couleur d'accent").strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let mut color = theme::accent();
                if ui.color_edit_button_srgba(&mut color).changed() {
                    theme::set_accent(ui.ctx(), color);
                }
                if ui.button("Par défaut").clicked() {
                    theme::set_accent(ui.ctx(), theme::DEFAULT_ACCENT);
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Raccourcis clavier").strong());
            ui.label(
                egui::RichText::new(
                    "Cliquez sur un raccourci puis appuyez sur la nouvelle combinaison. \
                     Échap annule la saisie.",
                )
                .small()
                .weak(),
            );
            ui.add_space(8.0);

            let listening = app.shortcuts_editor.as_ref().and_then(|e| e.listening);
            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .spacing([32.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    for action in ShortcutAction::ALL {
                        ui.label(action.label());
                        let text = if listening == Some(action) {
                            "Appuyez sur une touche…".to_owned()
                        } else {
                            app.shortcuts
                                .binding_text(action)
                                .unwrap_or_else(|| "—".to_owned())
                        };
                        if ui.button(text).clicked() {
                            app.shortcuts_editor = Some(ShortcutsEditor {
                                listening: Some(action),
                            });
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(8.0);
            if ui.button("Réinitialiser").clicked() {
                app.shortcuts.reset();
                app.shortcuts.save();
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Applications externes").strong());
            ui.label(
                egui::RichText::new(
                    "Boutons ajoutés à la barre de statut. Le dossier courant est passé \
                     en argument à l'exécutable.",
                )
                .small()
                .weak(),
            );
            ui.add_space(4.0);
            let mut changed = false;
            let mut remove: Option<usize> = None;
            for (idx, external) in app.external_apps.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(
                            egui::TextEdit::singleline(&mut external.name)
                                .desired_width(90.0)
                                .hint_text("Nom"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::TextEdit::singleline(&mut external.command)
                                .desired_width(230.0)
                                .hint_text("Chemin de l'exécutable"),
                        )
                        .changed();
                    if ui.small_button("✖").clicked() {
                        remove = Some(idx);
                    }
                });
            }
            if let Some(idx) = remove {
                app.external_apps.remove(idx);
                changed = true;
            }
            if ui.button("+ Ajouter").clicked() {
                app.external_apps.push(Default::default());
                changed = true;
            }
            if changed {
                crate::fs_ops::save_external_apps(&app.external_apps);
            }

            if let Some(action) = listening {
                let captured = ui.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } => Some((*key, *modifiers)),
                        _ => None,
                    })
                });
                if let Some((key, modifiers)) = captured {
                    if key != egui::Key::Escape {
                        app.shortcuts
                            .set(action, egui::KeyboardShortcut::new(modifiers, key));
                        app.shortcuts.save();
                    }
                    if let Some(editor) = &mut app.shortcuts_editor {
                        editor.listening = None;
                    }
                }
            }
        });
    if !open {
        app.shortcuts_editor = None;
    }
}
