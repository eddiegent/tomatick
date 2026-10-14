---
title: Guide d'utilisation
sidebar_position: 2
---

# Guide d'utilisation

Tomatick est un simple minuteur. Il n'y a ni liste de tâches ni compte. Cette page présente tout ce que vous pouvez faire avec.

:::note À propos des images
Les images de cette documentation ont été générées à partir de l'interface avec des données d'exemple. Ce ne sont pas des captures d'un vrai bureau, et elles seront remplacées.
:::

## Le widget

Le widget est une tomate avec le temps restant au centre. Il n'a pas de cadre de fenêtre, reste au-dessus de vos autres fenêtres, et vous pouvez le déplacer en le saisissant par le corps. Tomatick se souvient de l'endroit où vous l'avez posé, écran compris. Si cet écran n'est plus là au prochain démarrage, le widget passe sur l'écran principal.

![Le widget pendant une phase de travail](/img/screenshots/fr/widget-work.png)

La couleur indique ce qui se passe, et un anneau autour de la tomate montre où vous en êtes dans la phase.

| Couleur | Signification |
|---|---|
| Rouge | Travail |
| Vert | Pause courte |
| Bleu | Pause longue |
| Ambre | Minuteur |

![Pause courte](/img/screenshots/fr/widget-short-break.png)

Le petit nombre au-dessus du temps, comme `2/4`, indique la position dans le cycle. Choisissez une taille dans les Réglages : petit (140 px), moyen (200 px) ou grand (280 px). Aux tailles moyenne et grande, l'exercice d'une pause est affiché sous le temps. Au petit format, il est dans l'infobulle.

### Les boutons

Passez la souris sur le widget : les boutons apparaissent en fondu autour de la tomate.

![Le widget avec ses boutons](/img/screenshots/fr/widget-controls.png)

| Bouton | Action |
|---|---|
| Lecture et pause | Démarre la phase, la met en pause, ou la reprend |
| Passer | Termine la phase tout de suite et passe à la suivante (mode Pomodoro uniquement) |
| Réinitialiser | Remet la phase en cours à sa durée complète, sans la démarrer |
| Mode | Bascule entre Pomodoro et minuteur |
| Graphique | Ouvre les statistiques |
| Roue dentée | Ouvre les réglages |

Un minuteur en pause est atténué et affiche un symbole de pause. Si c'est l'ordinateur qui l'a mis en pause en se mettant en veille, une petite lune s'affiche aussi, pour que vous sachiez que ce n'est pas vous.

![Un minuteur en pause](/img/screenshots/fr/widget-paused.png)

## Mode Pomodoro

Un cycle est une série de phases de travail, chacune suivie d'une pause courte, sauf la dernière, qui est suivie d'une pause longue. Le réglage par défaut est le cycle classique : 25 minutes de travail, 5 minutes de pause courte, 15 minutes de pause longue, une pause longue après 4 phases de travail. Le bouton **Revenir au classique** des Réglages recharge ces valeurs.

- **Pause** arrête le minuteur aussi longtemps que vous le souhaitez, pour le déjeuner ou un appel.
- **Le démarrage automatique** est désactivé par défaut. Désactivé, le minuteur attend que vous appuyiez sur lecture après chaque phase. Activé, la phase suivante démarre toute seule.
- **Passer** une phase de travail ne la compte pas comme un pomodoro terminé, mais vous fait avancer dans le cycle : en passant plusieurs fois de suite, vous arrivez quand même à la pause longue.
- **Réinitialiser** une phase déjà démarrée l'enregistre dans l'historique comme interrompue.
- Si l'application se ferme ou plante pendant une phase, celle-ci est enregistrée comme interrompue au prochain démarrage. Une phase à moitié terminée n'est pas reprise.

### Les exercices de pause

Gardez une liste d'exercices dans les Réglages (gainage, abdominaux, étirements), chacun avec une quantité facultative comme `30 s` ou `15 répétitions`. Chaque pause affiche l'exercice suivant de la liste, et la liste recommence quand elle est terminée. La position est conservée d'une session à l'autre. Avec une liste vide, les pauses n'ont pas d'intitulé.

Une pause longue est découpée en plusieurs parties, chacune aussi longue qu'une pause courte, et chaque partie a son propre exercice. Une pause longue de 20 minutes avec des pauses courtes de 5 minutes donne 4 parties et 4 exercices à la suite. Si la pause longue n'est pas un multiple de la pause courte, le reste va à la dernière partie. Un son est joué quand la pause passe à la partie suivante.

![La pause longue](/img/screenshots/fr/widget-long-break.png)

## Mode minuteur

Le mode minuteur sert pour le thé, les pâtes ou une tâche rapide. Il n'y a qu'un minuteur à la fois, il est affiché en ambre, et il n'alerte qu'une fois, à zéro. Les minuteurs ne sont pas enregistrés dans les statistiques.

![Le minuteur](/img/screenshots/fr/widget-countdown.png)

Quand le minuteur n'est pas lancé, cliquez sur le temps pour saisir une nouvelle durée. Tapez des minutes (`10`) ou des minutes et des secondes (`1:30`), puis Entrée. Des choix rapides sont aussi proposés, `5`, `10`, `15` et `30` minutes par défaut, que vous pouvez changer dans les Réglages.

Un seul mode est actif à la fois. Changer de mode remet à zéro le minuteur que vous quittez : une phase Pomodoro déjà démarrée est enregistrée comme interrompue, et le minuteur revient à sa durée réglée. Vos réglages ne sont pas modifiés.

## Alertes

Pour chacun des quatre événements, vous choisissez ce qui se passe : une notification système, un son, une superposition plein écran, et la mise au premier plan du widget.

| Événement | Quand |
|---|---|
| Fin du travail | Une pause commence |
| Fin de la pause courte | C'est l'heure de travailler |
| Fin de la pause longue | C'est l'heure de travailler |
| Fin du minuteur | Le minuteur arrive à zéro |

### La superposition plein écran

La superposition assombrit l'écran qui contient le widget et affiche la phase, le temps restant et l'exercice.

![La superposition de pause](/img/screenshots/fr/overlay-break.png)

Pendant une pause, vous pouvez :

- la **fermer** (Échap ou le bouton). La pause continue sur le widget ;
- **passer la pause**, ce qui la termine et démarre la phase de travail suivante ;
- **prolonger** la pause de 1 ou 5 minutes. Le temps ajouté est conservé dans l'historique.

À la fin d'un minuteur, la superposition affiche « Temps écoulé », avec **Fermer** et **Relancer**.

Il n'y a pas de mode strict : la superposition ne vous enferme jamais. Les Réglages ont un bouton **Aperçu de la superposition** pour la voir avec un contenu d'exemple, sans toucher au minuteur.

### Les sons

Quatre sons sont fournis : un carillon doux, une cloche, un bip discret et un marimba. Vous pouvez aussi ajouter votre propre fichier MP3, WAV ou OGG pour n'importe quel événement. Le fichier est copié dans le dossier de données de Tomatick, il continue donc de fonctionner si vous déplacez ou supprimez l'original. Le curseur de volume et l'option « couper » s'appliquent à toutes les alertes et ne dépendent pas du volume du système. Chaque son a un bouton **Essayer**.

## L'icône de la zone de notification

Le menu de l'icône affiche ou masque le widget, démarre, met en pause ou reprend le minuteur, passe à la phase suivante, change de mode, ouvre les statistiques et les réglages, et quitte l'application. Quand une version plus récente existe, une ligne qui l'indique apparaît en haut. Un clic gauche sur l'icône affiche ou masque le widget (Windows et macOS). Pour quitter, utilisez **Quitter Tomatick** dans le menu.

## Raccourcis clavier

Trois raccourcis globaux fonctionnent même quand une autre application a le focus. Sous macOS, remplacez `Ctrl` par `Cmd`.

| Action | Par défaut |
|---|---|
| Démarrer, mettre en pause ou reprendre | `Ctrl+Alt+P` |
| Passer à la phase suivante | `Ctrl+Alt+N` |
| Afficher ou masquer le widget | `Ctrl+Alt+T` |

Vous pouvez les modifier dans les Réglages, section Système : cliquez sur un raccourci, puis tapez les nouvelles touches. Un raccourci doit comporter au moins une touche de modification (Ctrl, Alt, Maj ou Cmd). Si une combinaison est déjà utilisée par une autre application ou par votre système, les Réglages vous le disent. Sous Wayland, les raccourcis globaux peuvent ne pas fonctionner, voir [Installation](./installation.md).

## Veille et verrouillage

Les Réglages permettent de choisir ce que fait le minuteur quand l'ordinateur se met en veille :

- **Mettre le minuteur en pause** (par défaut). Le minuteur se met en pause et le reste à votre retour, jusqu'à ce que vous le repreniez. Le temps passé en pause est conservé dans l'historique.
- **Laisser le minuteur tourner.** Le minuteur suit l'horloge. Une phase peut se terminer pendant votre absence, et ses alertes sont données au réveil de l'ordinateur.

:::caution La veille est détectée, le verrouillage de l'écran ne l'est pas
Tomatick remarque que l'ordinateur s'est mis en veille, mais il ne sait pas encore que vous avez verrouillé l'écran. Le réglage dit « se met en veille ou se verrouille », et le verrouillage est prévu. Pour l'instant, verrouiller l'écran sans mise en veille ne met pas le minuteur en pause.
:::

## Statistiques

L'icône de graphique ouvre la fenêtre des statistiques. Elles ne concernent que le mode Pomodoro.

![La fenêtre des statistiques](/img/screenshots/fr/statistics.png)

- **Aujourd'hui** montre vos pomodoros terminés, le temps de concentration et votre progression vers l'objectif quotidien (8 par défaut).
- Les vues **jour, semaine et mois** affichent des graphiques en barres des pomodoros ou du temps de concentration, avec des boutons précédent et suivant.
- **La série** compte les jours consécutifs avec au moins un pomodoro terminé, actuelle et record.
- **Le taux de réussite** compare les phases de travail terminées à celles qui ont été passées ou interrompues.
- **La meilleure heure de la journée** montre à quelles heures vous terminez le plus de pomodoros.
- **Les totaux** comprennent les jours actifs et le temps de concentration moyen par jour actif.

Un pomodoro est une phase de travail terminée. Le temps de concentration est le temps travaillé, sans le temps passé en pause. Les jours suivent votre fuseau horaire local, et la semaine commence le jour indiqué par les paramètres régionaux de votre système.

Tout est stocké sur votre ordinateur, sans limite de durée. Cette version n'a ni export ni import.

## Langue

Tomatick parle anglais et français. Par défaut, il suit la langue de votre système, et vous pouvez en choisir une autre dans les Réglages. Le menu de la zone de notification et les notifications changent de langue aussi. Pour ajouter une langue, voir le [guide de traduction](./translation.md).
