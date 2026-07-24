//! Bandeau de filtres rapides (type, taille, date), sous la toolbar.
//! Les filtres appartiennent au volet actif et se réinitialisent en naviguant.

use crate::app::App;
use crate::fs_ops::FileKind;
use crate::pane::{DateFilter, SizeFilter};

pub(crate) fn show(app: &mut App, ui: &mut egui::Ui) {
    if !app.show_filters {
        return;
    }
    let frame =
        egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(12, 8));
    egui::Panel::top("filters").frame(frame).show(ui, |ui| {
        ui.horizontal(|ui| {
            let pane = app.pane_mut();

            egui::ComboBox::from_id_salt("filter_kind")
                .selected_text(pane.filter_kind.map_or("Type : tous", FileKind::label))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut pane.filter_kind, None, "Tous");
                    for kind in FileKind::FILTERABLE {
                        ui.selectable_value(&mut pane.filter_kind, Some(kind), kind.label());
                    }
                });

            egui::ComboBox::from_id_salt("filter_size")
                .selected_text(pane.filter_size.label())
                .show_ui(ui, |ui| {
                    for variant in SizeFilter::VARIANTS {
                        ui.selectable_value(&mut pane.filter_size, variant, variant.label());
                    }
                });

            egui::ComboBox::from_id_salt("filter_date")
                .selected_text(pane.filter_date.label())
                .show_ui(ui, |ui| {
                    for variant in DateFilter::VARIANTS {
                        ui.selectable_value(&mut pane.filter_date, variant, variant.label());
                    }
                });

            if pane.has_filters() && ui.button("✖ Réinitialiser").clicked() {
                pane.clear_filters();
            }
        });
    });
}
