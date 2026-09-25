# Aurora

Un explorateur de fichiers pour Windows, écrit en Rust, pensé pour être léger,
rapide à faire évoluer, et suffisamment moderne dans son fonctionnement pour
remplacer l'Explorateur Windows au quotidien.

> Projet personnel en cours de développement — pas encore utilisable en
> production. Voir [Statut du projet](#statut-du-projet) plus bas.

## Pourquoi ce projet

L'Explorateur Windows fait le travail, mais je voulais quelque chose de plus
rapide à faire évoluer avec mes propres besoins (intégrations terminal/VS Code,
vue divisée, thème personnalisé...) sans les contraintes d'une grosse
application legacy. Rust + une UI native légère (egui) permettent d'avoir un
outil qui démarre vite, consomme peu de RAM, et reste simple à modifier.

## Stack technique

- **Rust** pour l'intégralité de la logique (fichiers, opérations, intégration
  Windows)
- **egui / eframe** pour l'interface, en immediate-mode — pas de JS, pas de
  WebView, un seul langage du début à la fin
- **[windows-rs](https://github.com/microsoft/windows-rs)** pour les appels
  API Windows bas niveau (icônes système, lecteurs, etc.)
- Crates notables : `trash` (suppression vers la corbeille), `open`
  (ouverture avec l'appli par défaut), `walkdir` (recherche récursive), `image`
  (conversion d'icônes)

Un premier prototype avait été fait en **Tauri** (Rust + WebView/HTML/CSS/JS)
pour valider rapidement l'idée ; le projet est reparti sur une base 100% Rust
(egui) pour prioriser la légèreté et la vélocité de développement sur le long
terme.

## Fonctionnalités

### Navigation

- Double-clic, dossier parent, précédent/suivant, breadcrumb cliquable
- Barre d'adresse éditable (Ctrl+L)
- Écran d'accueil : accès rapide (Bureau, Documents, Téléchargements...) et
  lecteurs avec jauge d'utilisation du disque
- **Onglets** (Ctrl+T / Ctrl+W / Ctrl+Tab), chacun avec son propre état
- **Vue divisée** (Ctrl+D) : deux volets indépendants côte à côte

### Affichage

- Vue liste détaillée avec tri par colonnes (nom, taille, date...)
- Vue grille avec tuiles colorées et miniatures d'images (chargées en
  arrière-plan, mises en cache)
- Icônes système réelles par type de fichier
- Rendu performant sur les gros dossiers (liste virtualisée, chargement
  asynchrone)
- Thème clair/sombre et couleur d'accent personnalisable

### Fichiers

- Créer un dossier, renommer (F2), supprimer (vers la corbeille), ouvrir avec
  l'application par défaut
- Multi-sélection (clic, Ctrl+clic, Shift+clic, Ctrl+A) et
  copier/couper/coller
- Glisser-déposer interne (Ctrl pour copier) et depuis l'Explorateur Windows
- Recherche par nom dans le dossier courant, plus filtres rapides par type,
  taille et date

### Intégrations et personnalisation

- Ouvrir le dossier courant dans un terminal (Windows Terminal / PowerShell /
  cmd) ou dans VS Code
- Liste d'applications externes configurable, avec boutons dans la barre
  d'état
- Raccourcis clavier configurables (éditeur intégré, stockés dans
  `%APPDATA%urora\shortcuts.conf`)

### Et ensuite

Le détail, découpé en petites étapes, est dans
[`docs/plan-implémentation.md`](docs/plan-impl%C3%A9mentation.md) et la liste
complète des fonctionnalités envisagées (y compris celles pas encore
planifiées) dans
[`docs/fonctionnalites-explorateur.md`](docs/fonctionnalites-explorateur.md).

## Statut du projet

🚧 En développement actif, usage personnel. Pas encore de release stable.
Windows uniquement pour l'instant (le code s'appuie directement sur des API
Win32 pour les icônes et les lecteurs).

## Installation (développement)

Prérequis :
- [Rust](https://rustup.rs) 1.85 ou plus récent (édition 2024, toolchain stable)
- Windows 10/11

```powershell
git clone <url-du-repo>
cd aurora
cargo run
```

## Build release

```powershell
cargo build --release
```

L'exécutable se trouve ensuite dans `target/release/aurora.exe`.

## Installeur

Un installeur NSIS peut être généré avec
[`cargo-packager`](https://crates.io/crates/cargo-packager) (la configuration
est dans `Cargo.toml`) :

```powershell
cargo install cargo-packager --locked   # une seule fois
cargo packager --release
```

Cela produit un `Aurora_x.y.z_x64-setup.exe`.

## Licence

Copyright © 2026 D4rkL4s3r — tous droits réservés.

Le code source est publié pour consultation uniquement. Toute utilisation,
copie, modification ou redistribution, en tout ou partie, nécessite mon accord
écrit préalable. Voir le fichier [`LICENSE`](LICENSE).
