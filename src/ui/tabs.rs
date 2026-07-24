use crate::app::App;
use crate::shortcuts::ShortcutAction;

/// Barre d'onglets au-dessus de la toolbar. Clic pour activer, clic molette
/// ou ✖ pour fermer, ＋ pour ouvrir un nouvel onglet sur le dossier courant.
pub(crate) fn show(app: &mut App, ui: &mut egui::Ui) {
    let frame =
        egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(10, 6));
    egui::Panel::top("tabs").frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            let mut activate: Option<usize> = None;
            let mut close: Option<usize> = None;
            let closable = app.tabs.len() > 1;
            for (idx, tab) in app.tabs.iter().enumerate() {
                let selected = idx == app.active_tab;
                let title = if selected {
                    egui::RichText::new(tab.title()).strong()
                } else {
                    egui::RichText::new(tab.title())
                };
                let response = ui.selectable_label(selected, title);
                if response.clicked() {
                    activate = Some(idx);
                }
                if response.middle_clicked() {
                    close = Some(idx);
                }
                if closable
                    && selected
                    && ui
                        .small_button("✖")
                        .on_hover_text(
                            app.shortcuts
                                .menu_label("Fermer l'onglet", ShortcutAction::CloseTab),
                        )
                        .clicked()
                {
                    close = Some(idx);
                }
            }
            if ui
                .button("＋")
                .on_hover_text(app.shortcuts.menu_label("Nouvel onglet", ShortcutAction::NewTab))
                .clicked()
            {
                app.new_tab();
            }
            if let Some(idx) = close {
                app.close_tab(idx);
            } else if let Some(idx) = activate {
                app.active_tab = idx;
            }
        });
    });
}
