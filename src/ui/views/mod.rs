//! Zone centrale : rendu des volets (un seul ou vue divisée), chaque volet
//! affichant l'accueil, la grille ou la table, et servant de cible de dépôt.

mod grid;
mod interactions;
mod table;

use crate::actions::{DragPayload, UiAction};
use crate::app::App;
use crate::pane::Pane;
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
        egui::Stroke::new(1.2, theme::accent().gamma_multiply(0.6))
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };
    egui::Frame::default()
        .stroke(stroke)
        .corner_radius(10.0)
        .inner_margin(8.0)
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
    if pane.is_home() {
        return; // pas de dossier cible sur l'écran d'accueil
    }
    let response = ui.response();
    if response.dnd_hover_payload::<DragPayload>().is_some() {
        ui.painter().rect_stroke(
            ui.max_rect(),
            6.0,
            egui::Stroke::new(1.5, theme::accent().gamma_multiply(0.6)),
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

/// Contenu d'un volet : accueil, chargement, erreur, sinon grille ou table.
fn show_content(app: &App, pane_idx: usize, ui: &mut egui::Ui, actions: &mut Vec<UiAction>) {
    let pane = &app.tab().panes[pane_idx];

    if pane.is_home() {
        crate::ui::home::show(app, ui, actions);
        return;
    }
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
    let displayed_idx: Vec<usize> = pane.displayed_indices();

    if app.grid_view {
        grid::show(app, pane, ui, base, &displayed_idx, actions);
    } else {
        table::show(app, pane, ui, base, &displayed_idx, actions);
    }
}
