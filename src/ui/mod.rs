//! Rendu egui, découpé par zone d'écran. Les fonctions de rendu ne modifient
//! jamais l'état directement : elles émettent des [`crate::actions::UiAction`].

pub(crate) mod dialogs;
pub(crate) mod filters;
pub(crate) mod format;
pub(crate) mod home;
pub(crate) mod icons;
pub(crate) mod settings;
pub(crate) mod sidebar;
pub(crate) mod statusbar;
pub(crate) mod tabs;
pub(crate) mod theme;
pub(crate) mod thumbs;
pub(crate) mod views;
pub(crate) mod toolbar;
