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

### Déjà là (ou en cours d'implémentation immédiate)

- Navigation dans les dossiers (double-clic, dossier parent, précédent,
  breadcrumb)
- Liste des lecteurs et raccourcis "Accès rapide" (Bureau, Documents,
  Téléchargements...)
- Opérations de base : créer un dossier, renommer, supprimer (corbeille),
  ouvrir avec l'application par défaut
- Multi-sélection (clic, Ctrl+clic, Shift+clic, Ctrl+A) et
  copier/couper/coller
- Recherche par nom dans le dossier courant
- Icônes système réelles par type de fichier
- Rendu performant même sur des dossiers volumineux (liste virtualisée,
  chargement asynchrone)

### Prévu ensuite

- Glisser-déposer interne et depuis l'Explorateur Windows natif
- Ouverture rapide du dossier courant dans un terminal (Windows Terminal /
  PowerShell / cmd) ou dans VS Code
- Vue divisée (deux volets côte à côte dans la même fenêtre)
- Vue liste détaillée avec tri par colonnes
- Thème clair/sombre

Le détail complet, découpé en petites étapes, est dans
[`docs/plan-implementation.md`](docs/plan-implementation.md) et la liste
complète des fonctionnalités envisagées (y compris celles pas encore
planifiées) dans [`docs/fonctionnalites.md`](docs/fonctionnalites.md).

## Statut du projet

🚧 En développement actif, usage personnel. Pas encore de release stable.
Windows uniquement pour l'instant (le code s'appuie directement sur des API
Win32 pour les icônes et les lecteurs).

## Installation (développement)

Prérequis :
- [Rust](https://rustup.rs) (édition 2021, toolchain stable)
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

L'exécutable se trouve ensuite dans `target/release/`.

## Licence

Projet personnel, pas encore de licence définie.
