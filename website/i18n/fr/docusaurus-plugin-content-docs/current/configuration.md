---
title: Référence de configuration
sidebar_position: 3
---

# Référence de configuration

Ouvrez les réglages avec la roue dentée du widget ou depuis le menu de la zone de notification. Cliquez sur **Enregistrer** pour appliquer vos changements, ou sur **Annuler** pour les abandonner. Une valeur hors de la plage autorisée est refusée, avec un message qui donne la plage.

![La fenêtre des réglages](/img/screenshots/settings.png)

## Cycle Pomodoro

| Réglage | Par défaut | Valeurs permises |
|---|---|---|
| Travail (minutes) | 25 | 1 à 180 |
| Pause courte (minutes) | 5 | 1 à 60 |
| Pause longue (minutes) | 15 | 1 à 120 |
| Séances de travail avant une pause longue | 4 | 2 à 12 |
| Objectif quotidien (pomodoros) | 8 | 1 à 50 |
| Démarrer automatiquement la phase suivante | Désactivé | Activé ou désactivé |
| Quand l'ordinateur se met en veille ou se verrouille | Mettre le minuteur en pause | Mettre en pause, ou laisser tourner |

**Revenir au classique** charge 25, 5, 15 et 4 séances dans le formulaire. Il faut encore enregistrer pour les appliquer.

## Widget

La taille du widget : petit (140 px), moyen (200 px, par défaut) ou grand (280 px).

## Minuteur

| Réglage | Par défaut | Valeurs permises |
|---|---|---|
| Valeur de départ | 10 minutes | 0:01 à 23:59:59 |
| Choix rapides | 5, 10, 15, 30 minutes | Une à six durées |

Saisissez les durées en minutes (`10`) ou en minutes et secondes (`1:30`). Séparez les choix rapides par des virgules. Ils apparaissent quand vous cliquez sur le temps du widget pour modifier le minuteur.

## Alertes

Un tableau permet de choisir, pour chacun des quatre événements, ce qui se passe.

![Les réglages d'alertes](/img/screenshots/settings-alerts.png)

| Option | Effet |
|---|---|
| Notification système | Une notification native, avec l'exercice s'il y en a un |
| Son | Joue le son choisi pour cet événement |
| Superposition plein écran | Assombrit l'écran avec la phase, le temps restant et l'exercice |
| Amener le widget au premier plan | Place le widget au-dessus des autres fenêtres |

Par défaut, chaque événement a une notification et un son, et la superposition est activée à la fin d'une phase de travail et à la fin d'un minuteur.

Les autres options de cette section :

- **Son** pour chaque événement : un son fourni (carillon doux, cloche, bip discret, marimba) ou **Ajouter un fichier** pour utiliser votre propre fichier MP3, WAV ou OGG. Par défaut : la cloche à la fin du travail, le carillon à la fin d'une pause courte, le marimba à la fin d'une pause longue et le bip à la fin d'un minuteur.
- **Volume**, de 0 à 100, pour tous les sons. Il est indépendant du volume du système.
- **Couper tous les sons d'alerte.**
- Des boutons **Essayer** pour écouter un son.
- **Aperçu de la superposition** pour la voir sans toucher au minuteur.

## Exercices de pause

Une liste ordonnée. Pour chaque entrée, donnez un nom et une quantité facultative (`30 s`, `15 répétitions`). Faites glisser la poignée pour réordonner, ou utilisez les boutons monter et descendre, qui fonctionnent aussi au clavier. Voir les [exercices de pause](./user-guide.md#les-exercices-de-pause).

## Système

| Réglage | Par défaut |
|---|---|
| Lancer Tomatick à l'ouverture de session | Désactivé |
| Raccourcis globaux | `Ctrl+Alt+P` démarrer ou pause, `Ctrl+Alt+N` passer, `Ctrl+Alt+T` afficher ou masquer (`Cmd` sous macOS) |
| Langue | Comme le système |
| Rechercher les nouvelles versions | Activé |

Si un raccourci ne peut pas être enregistré, ou si le lancement à l'ouverture de session ne peut pas être modifié, le réglage est enregistré et une erreur s'affiche dans cette section. Chaque raccourci a des boutons **Effacer** et **Par défaut**.

## À propos

Affiche la version, la licence, un lien vers le code source et un lien vers cette documentation. Indique aussi s'il existe une version plus récente, et **Vérifier maintenant** interroge GitHub tout de suite, même si la recherche automatique est désactivée.
