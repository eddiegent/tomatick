---
title: Architecture
sidebar_position: 6
---

# Architecture

Tomatick est une application [Tauri 2](https://tauri.app). Un cœur en Rust fait le travail et une interface React, écrite en TypeScript, l'affiche dans la vue web du système d'exploitation. Aucun navigateur n'est embarqué.

```text
tomatick/
  src-tauri/          Cœur en Rust
  src/                Interface React
  locales/            Fichiers de traduction (partagés par les deux côtés)
  tests/              Tests de l'interface, tests de bout en bout, images de la documentation
  website/            Ce site
  .github/workflows/  CI, constructions des versions, déploiement de la doc
```

## La règle

**Le moteur en Rust est la seule source de vérité.** L'interface se contente d'afficher l'état qu'elle reçoit et d'envoyer des commandes. Rien dans l'interface ne compte le temps.

## Le cœur en Rust

| Module | Rôle |
|---|---|
| `engine` | La machine à états et la logique du cycle. Du Rust pur, avec une horloge injectable pour que les tests n'aient pas à attendre |
| `storage` | SQLite : migrations, réglages, exercices et leur rotation |
| `history` | L'enregistrement des phases et les requêtes de statistiques |
| `alerts` | Relie les événements du moteur à une notification, un son, la superposition et la mise au premier plan |
| `overlay` | Les fenêtres de superposition plein écran |
| `placement` | La mémoire de la position du widget et de l'écran où il se trouve |
| `power` | Détecter que l'ordinateur s'est mis en veille |
| `shortcuts` | Validation et enregistrement des raccourcis globaux |
| `tray` | L'icône de la zone de notification et son menu |
| `updater` | La recherche d'une version plus récente |
| `i18n` | Les traductions du menu, des notifications et des titres de fenêtres |
| `lib.rs` | Les commandes Tauri, les événements et la boucle qui fait tout avancer |

Le moteur a quatre états : au repos, en marche, en pause par l'utilisateur, et en pause par le système. Il calcule le temps restant à partir d'une horloge monotone et d'horodatages, jamais en comptant des ticks, donc le temps reste juste même si l'interface est ralentie ou si l'ordinateur est chargé.

Un fil d'exécution appelle le moteur toutes les 250 ms. Il transforme ce que le moteur signale en lignes d'historique, en alertes et en événements, et il détecte quand l'ordinateur s'est mis en veille.

## Dialogue avec l'interface

L'interface appelle des commandes Tauri (`invoke`) et écoute des événements.

| Événement | Quand |
|---|---|
| `engine:state` | Après chaque commande, et pendant qu'un minuteur tourne. Le contenu est un instantané : mode, phase, temps restant, partie, intitulé, position dans le cycle |
| `engine:event` | Une phase a été enregistrée, une alerte est due, ou une pause longue passe à sa partie suivante |
| `settings:changed` | Les réglages ont été enregistrés |
| `history:changed` | Une phase a été ajoutée à l'historique |
| `update:changed` | Une recherche de mise à jour s'est terminée |

## L'interface

Un seul paquet sert toutes les fenêtres. C'est l'étiquette de la fenêtre qui décide de ce qui est affiché.

| Dossier | Fenêtre |
|---|---|
| `src/widget` | La tomate |
| `src/settings` | Les réglages |
| `src/stats` | Les statistiques |
| `src/overlay` | La superposition plein écran |
| `src/i18n` | La configuration d'i18next, partagée par toutes |

## Les données

Un seul fichier SQLite, `tomatick.db`, dans le dossier de données de l'application. Les migrations sont versionnées et s'exécutent au démarrage. Les réglages sont stockés en JSON dans une table `settings`, la liste d'exercices dans `exercises`, et chaque phase dans `phases`, avec son statut (terminée, passée ou interrompue) et les temps passés en pause et ajoutés. Les horodatages sont en UTC, et les jours sont regroupés selon le fuseau horaire local au calcul des statistiques. Les statistiques viennent de requêtes de regroupement, sans table pré-calculée.

## Les traductions

Les fichiers de `locales` sont lus par l'interface à la compilation et intégrés au cœur en Rust par `build.rs`, si bien que le menu et les notifications utilisent les mêmes textes que les fenêtres.

## Réseau

La seule requête réseau est la recherche de mises à jour : `updater` interroge l'API des versions de GitHub pour connaître la dernière version et compare les numéros. Il ne télécharge jamais rien.

## Tests

- Des tests unitaires en Rust, pour le moteur, le stockage et les statistiques.
- Des tests Playwright de l'interface contre un faux backend (`tests/ui`).
- Des tests de bout en bout de la vraie application avec `tauri-driver` (`tests/e2e`).
- Une construction CI sous Windows, macOS et Linux pour chaque pull request.

## Versions

Pousser une étiquette qui commence par `v` lance le workflow de publication. Il vérifie que l'étiquette correspond à la version des fichiers du projet, construit les installateurs sur les trois systèmes, et crée un brouillon de version sur GitHub que le mainteneur relit et publie.
