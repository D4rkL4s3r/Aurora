//! Interactions communes aux entrées de la grille et de la table :
//! sélection, ouverture, menu contextuel et glisser-déposer.

use crate::actions::{DragPayload, UiAction};
use crate::app::App;
use crate::fs_ops::FileEntry;
use crate::pane::Pane;
use crate::ui::theme;

fn entry_context_menu(
    app: &App,
    entry: &FileEntry,
    response: &egui::Response,
    actions: &mut Vec<UiAction>,
) {
    use crate::shortcuts::ShortcutAction;
    let shortcuts = &app.shortcuts;
    response.context_menu(|ui| {
        if !entry.is_dir && ui.button("Ouvrir").clicked() {
            actions.push(UiAction::OpenFile(entry.path.clone()));
            ui.close();
        }
        if ui
            .button(shortcuts.menu_label("Renommer", ShortcutAction::Rename))
            .clicked()
        {
            actions.push(UiAction::StartRename(entry.path.clone()));
            ui.close();
        }
        ui.separator();
        if ui
            .button(shortcuts.menu_label("Copier", ShortcutAction::Copy))
            .clicked()
        {
            actions.push(UiAction::CopySelection { cut: false });
            ui.close();
        }
        if ui
            .button(shortcuts.menu_label("Couper", ShortcutAction::Cut))
            .clicked()
        {
            actions.push(UiAction::CopySelection { cut: true });
            ui.close();
        }
        ui.separator();
        if ui
            .button(shortcuts.menu_label("Supprimer", ShortcutAction::Delete))
            .clicked()
        {
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
        egui::Stroke::new(1.5, theme::accent()),
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

pub(super) fn entry_interactions(
    app: &App,
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
    entry_context_menu(app, entry, response, actions);
}
