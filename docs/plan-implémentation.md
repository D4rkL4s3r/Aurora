# Plan d'implémentation — Aurora (explorateur de fichiers Rust, egui/eframe)

Hypothèse de départ : **egui + eframe**, un seul langage (Rust), pas de Node/JS.
Chaque étape ci-dessous est volontairement petite et autonome : un ticket = une
fonctionnalité testable, livrable en une session de travail (avec Claude Code ou
en solo). Ne pas hésiter à sauter des étapes ⚪ si elles ne sont pas prioritaires.

Légende : 🔴 MVP · 🟡 V2 · ⚪ Stretch (reprend les priorités de la liste de fonctionnalités)

---

## Phase 0 — Socle du projet

**0.1** 🔴 Initialiser le projet `cargo new`, ajouter `eframe`/`egui` en dépendance,
afficher une fenêtre vide avec un titre.
*Critère de succès : `cargo run` ouvre une fenêtre native vide.*

**0.2** 🔴 Ajouter la structure de code : un module `fs_ops` (logique fichier, zéro
dépendance UI) et un module `app` (état + rendu egui). Définir la struct `App`
avec un champ `current_path: PathBuf`.
*Critère de succès : les deux modules compilent, séparés proprement.*

**0.3** 🔴 Définir la struct `FileEntry` (nom, chemin, is_dir, taille, date de
modif, extension) et une fonction `fs_ops::list_dir(path) -> Result<Vec<FileEntry>>`.
*Critère de succès : test unitaire qui liste le dossier courant et vérifie qu'il
n'est pas vide.*

---

## Phase 1 — Navigation de base

**1.1** 🔴 Afficher la liste des `FileEntry` du dossier courant sous forme de
liste simple (une ligne par fichier, nom + icône texte 📁/📄).
*Critère de succès : lancer l'app sur `C:\Users\...` affiche bien le contenu.*

**1.2** 🔴 Double-clic sur un dossier → navigue dedans (met à jour `current_path`
et recharge la liste). Double-clic sur un fichier → ne fait rien pour l'instant
(sera branché en 3.x).
*Critère de succès : on peut descendre dans l'arborescence par double-clic.*

**1.3** 🔴 Bouton "dossier parent" (remonter d'un niveau).

**1.4** 🔴 Bouton "précédent" avec pile d'historique simple (`Vec<PathBuf>`).

**1.5** 🟡 Breadcrumb cliquable (afficher le chemin découpé en segments cliquables
au lieu du texte brut).

**1.6** 🟡 Barre d'adresse éditable (champ texte qui affiche/accepte le chemin en
brut, avec validation à la touche Entrée).

---

## Phase 2 — Sidebar

**2.1** 🔴 Lister les lecteurs Windows (`GetLogicalDrives` via le crate `windows`,
même code que dans le prototype Tauri) et les afficher dans une colonne de
gauche, cliquables.

**2.2** 🟡 Accès rapide (Bureau, Documents, Téléchargements...) — réutiliser la
logique déjà écrite dans le prototype Tauri (`list_quick_access`), juste
renvoyée directement en Rust au lieu de passer par une commande IPC.

---

## Phase 3 — Opérations fichiers de base

**3.1** 🔴 Ouvrir un fichier avec l'application par défaut au double-clic
(crate `open`).

**3.2** 🔴 Créer un nouveau dossier (bouton toolbar + boîte de dialogue de saisie
du nom).

**3.3** 🔴 Renommer (menu clic droit ou touche F2 + boîte de dialogue).

**3.4** 🔴 Supprimer → corbeille (crate `trash`), avec confirmation.

**3.5** 🟡 Copier un fichier/dossier vers une destination (fonction déjà écrite,
juste à re-brancher).

**3.6** 🟡 Déplacer un fichier/dossier vers une destination.

---

## Phase 4 — Sélection et presse-papiers

**4.1** 🟡 Sélection simple (clic) avec surbrillance visuelle.

**4.2** 🟡 Multi-sélection : Ctrl+clic (ajout/retrait), Shift+clic (plage).

**4.3** 🟡 Ctrl+A (tout sélectionner dans le dossier courant).

**4.4** 🟡 Copier/Couper/Coller (Ctrl+C/X/V) sur la sélection courante, en
s'appuyant sur 3.5/3.6.

**4.5** 🟡 Suppr au clavier → corbeille sur la sélection.

---

## Phase 5 — Recherche

**5.1** 🔴 Champ de recherche dans la toolbar, recherche par nom limitée au
dossier courant (non récursive pour commencer, retour instantané).

**5.2** 🟡 Rendre la recherche récursive (profondeur limitée, ex. 8 niveaux) et
plafonner le nombre de résultats (ex. 300) pour rester réactif.

**5.3** ⚪ Filtres rapides (par type de fichier, par taille, par date).

---

## Phase 6 — Icônes et affichage

**6.1** 🔴 Icônes texte/emoji par catégorie (dossier, image, vidéo, audio,
archive, code, défaut) — rapide à faire, sert de fallback visuel pendant tout
le développement.

**6.2** 🟡 Vraies icônes système (extraction `SHGetFileInfo` + conversion GDI →
texture egui, avec cache par extension). *Étape la plus technique — isoler dans
son propre module comme dans le prototype Tauri, pour limiter le risque aux
autres fonctionnalités si ça bloque à la compilation.*

**6.3** 🟡 Vue liste détaillée avec colonnes triables (nom/taille/type/date),
en plus de la vue grille.

**6.4** ⚪ Miniatures réelles pour images (via crate `image`, en tâche de fond
pour ne pas bloquer l'UI).

---

## Phase 7 — Performance sur gros dossiers

**7.1** 🔴 Chargement asynchrone du contenu d'un dossier (thread séparé + canal
vers l'UI), pour ne jamais geler la fenêtre même sur un dossier réseau lent.

**7.2** 🔴 Liste virtualisée : utiliser `egui_extras::TableBuilder` avec
`show_rows` pour ne rendre que les lignes visibles à l'écran.
*Critère de succès : ouvrir un dossier avec 50 000+ fichiers reste fluide.*

**7.3** 🟡 Gestion propre des erreurs d'accès (dossier protégé, lecteur réseau
déconnecté) sans crash, avec message clair à l'utilisateur.

---

## Phase 8 — Glisser-déposer

**8.1** 🟡 Glisser-déposer interne (sélection → dossier cible dans la même
fenêtre) pour déplacer, Ctrl maintenu = copier.

**8.2** 🟡 Glisser-déposer natif (fichiers déposés depuis l'Explorateur Windows
dans la fenêtre de l'app → copie dans le dossier courant). *Vérifier le support
natif d'eframe/winit pour les événements de drop fichier — potentiellement le
point le plus incertain de cette phase.*

---

## Phase 9 — Intégrations externes

**9.1** 🟡 Bouton "Ouvrir un terminal ici" (détecter Windows Terminal, sinon
PowerShell, sinon cmd — lancer avec le dossier courant comme répertoire de
travail).

**9.2** 🟡 Bouton "Ouvrir dans VS Code" (détecter `code.exe` dans le PATH,
lancer avec le dossier courant en argument).

**9.3** ⚪ Liste configurable d'applications externes personnalisées (au-delà du
terminal/VS Code).

---

## Phase 10 — Vue divisée

**10.1** 🟡 Refactorer l'état courant (`current_path`, sélection, historique)
en une struct `Pane` réutilisable, dont `App` possède une instance.

**10.2** 🟡 Bouton "Diviser" qui affiche deux `Pane` côte à côte, chacun
indépendant (son propre chemin, sa propre sélection).

**10.3** ⚪ Glisser-déposer entre les deux volets (réutilise la logique de 8.1).

**10.4** ⚪ Onglets (généraliser `Pane` à une liste plutôt qu'un affichage figé
à 1 ou 2 volets).

---

## Phase 11 — Finitions

**11.1** 🟡 Thème sombre/clair, bascule manuelle ou suivi du thème système.

**11.2** ⚪ Personnalisation des couleurs d'accent.

**11.3** ⚪ Raccourcis clavier reconfigurables.

**11.4** 🔴 Packaging : build release optimisé, icône d'application, test sur
une machine Windows "propre" (sans toolchain Rust) pour vérifier qu'il n'y a
pas de DLL manquante.

---

## Notes d'usage de ce plan

- Les étapes 🔴 forment un MVP utilisable au quotidien dès la fin de la Phase 7
  (navigation + opérations de base + perf sur gros dossiers).
- Chaque étape peut être donnée telle quelle à Claude Code comme description de
  tâche ; elles sont volontairement scoping-serrées pour éviter qu'une session
  parte dans plusieurs directions à la fois.
- Le module `fs_ops` (Phase 0.2) doit rester indépendant d'egui autant que
  possible : ça permet de tester la logique fichier sans UI, et de la
  réutiliser telle quelle si jamais tu changes de framework UI plus tard.
