# Aurora — liste de fonctionnalités candidates

Légende priorité : 🔴 MVP (indispensable v1) · 🟡 V2 (important mais pas bloquant) · ⚪ Stretch (confort / plus tard)

## 1. Navigation

- 🔴 Navigation par dossiers (double-clic), dossier parent, précédent/suivant
- 🔴 Breadcrumb cliquable
- 🟡 Barre d'adresse éditable (taper/coller un chemin directement)
- 🟡 Historique de navigation persistant (liste des N derniers dossiers visités)
- ⚪ Onglets (plusieurs dossiers ouverts en parallèle)
- 🟡 Fenêtres divisées (deux volets côte à côte, glisser-déposer facilité entre les deux)
- ⚪ Favoris/épingles personnalisés dans la sidebar (en plus de l'Accès rapide)

## 2. Opérations sur les fichiers

- 🔴 Créer dossier / fichier, renommer, supprimer (corbeille)
- 🔴 Copier / Couper / Coller (clic droit, raccourcis, glisser-déposer)
- 🟡 Barre de progression pour les copies/déplacements volumineux, annulable
- 🟡 Undo/Redo sur les opérations (au moins renommer/déplacer/supprimer)
- 🟡 Gestion des conflits (fichier déjà existant → remplacer / ignorer / renommer / garder les deux)
- ⚪ Compression/décompression (zip) intégrée
- ⚪ Propriétés fichier (taille, dates, attributs, permissions) dans un panneau ou une modale

## 3. Affichage & vues

- 🔴 Vue grille (actuelle)
- 🟡 Vue liste détaillée avec colonnes triables (nom / taille / type / date modifiée)
- 🟡 Tri et regroupement (par type, par date...)
- ⚪ Miniatures réelles pour images/vidéos (pas juste l'icône du type de fichier)
- ⚪ Panneau de prévisualisation (aperçu d'un fichier sélectionné sans l'ouvrir)
- ⚪ Densité d'affichage réglable (compact / confortable)

## 4. Recherche & filtres

- 🔴 Recherche par nom dans le dossier courant (déjà en place)
- 🟡 Filtres rapides (par type de fichier, par taille, par date de modification)
- 🟡 Recherche insensible aux accents/casse, avec surlignage des résultats
- ⚪ Recherche plein texte dans le contenu des fichiers (via indexation ou Windows Search)
- ⚪ Requêtes sauvegardées / dossiers de recherche virtuels

## 5. Performance & robustesse

- 🔴 Liste virtualisée : ne rendre que les lignes visibles, fluide même sur des
  dossiers à dizaines de milliers d'entrées
- 🔴 Chargement asynchrone du contenu d'un dossier (jamais de gel de l'UI)
- 🟡 Chargement progressif des icônes/miniatures (déjà partiellement en place)
- 🟡 Gestion propre des dossiers inaccessibles (permissions refusées, lecteur réseau
  déconnecté) sans planter l'appli
- ⚪ Surveillance en direct du dossier (rafraîchissement auto si un fichier est
  ajouté/supprimé par un autre programme pendant que le dossier est ouvert)

## 6. Intégration Windows

- 🔴 Vraies icônes système par type de fichier
- 🟡 Glisser-déposer natif (avec l'Explorateur et d'autres applis) — déjà en place
- 🟡 Ouverture avec l'application par défaut — déjà en place
- ⚪ Menu contextuel natif du Shell (les extensions tierces comme 7-Zip, Git, antivirus
  qui ajoutent leurs propres entrées) — techniquement complexe (COM `IContextMenu`)
- ⚪ Intégration à la corbeille système (voir/restaurer depuis l'appli)
- ⚪ Association de fichiers / "Ouvrir avec..." (choisir parmi les applis installées)
- ⚪ Raccourcis (.lnk) : les résoudre et les afficher avec la bonne icône/cible

## 7. Personnalisation

- 🟡 Thème clair/sombre, voire suivre le thème système
- ⚪ Couleurs d'accent personnalisables
- ⚪ Raccourcis clavier reconfigurables
- ⚪ Disposition de la sidebar personnalisable (réordonner, masquer des sections)

## 8. Fonctions avancées / power-user

- 🟡 Multi-sélection avancée (déjà en place : clic/Ctrl/Shift/Ctrl+A)
- 🟡 Renommage en lot (pattern, numérotation automatique)
- 🟡 **Ouvrir le dossier courant dans une appli externe** : terminal (cmd, PowerShell,
  Windows Terminal), VS Code, ou toute autre appli configurable — bouton toolbar
  et/ou clic droit. Techniquement simple (lancer un process avec le chemin en
  argument), gros gain de confort pour un usage dev au quotidien.
- ⚪ Liste des applis "Ouvrir avec..." personnalisable (au-delà du terminal/VS Code,
  laisser ajouter n'importe quel exécutable avec ses arguments)
- ⚪ Ligne de commande intégrée directement dans la fenêtre (mini-terminal en bas,
  plutôt qu'une fenêtre externe séparée)
- ⚪ Calcul de la taille d'un dossier (récursif, en tâche de fond)
- ⚪ Comparaison de deux dossiers (diff de contenu)
- ⚪ Scripting/automatisation (actions personnalisées sur sélection de fichiers)

## 9. Sécurité & fiabilité

- 🔴 Suppression = corbeille par défaut, jamais de suppression définitive sans
  confirmation explicite
- 🟡 Confirmation avant opérations destructives sur un grand nombre de fichiers
- ⚪ Historique des actions récentes (pour audit personnel, pas juste undo immédiat)

---

**Prochaine étape suggérée** : valider ensemble la liste 🔴 (MVP), ajuster si besoin,
puis on s'en sert de cahier des charges pour trancher/valider le choix de techno
(egui a été proposé — à confirmer une fois le MVP figé) et démarrer l'implémentation.
