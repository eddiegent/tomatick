---
title: Questions fréquentes
sidebar_position: 4
---

# Questions fréquentes

## Tomatick a-t-il une liste de tâches ?

Non, et il n'en aura pas. C'est un simple minuteur : ni tâches, ni comptes, ni cloud.

## Envoie-t-il mes données quelque part ?

Non. L'historique et les réglages restent dans un fichier sur votre ordinateur. La seule requête réseau est la recherche de nouvelles versions, qui demande à GitHub la dernière version publiée. Vous pouvez la désactiver dans les Réglages.

## Puis-je exporter mon historique ?

Pas dans cette version. L'historique est un fichier SQLite, `tomatick.db`, dans le dossier de données de l'application (voir [Installation](./installation.md#où-sont-stockées-vos-données)), que vous pouvez lire avec n'importe quel outil SQLite.

## Windows ou macOS m'avertit à l'ouverture

Les installateurs ne sont pas signés avec un certificat payant, donc les deux systèmes avertissent d'un éditeur inconnu. Voir [Installation](./installation.md) pour savoir comment continuer.

## Le widget ne reste pas au premier plan, ou mes raccourcis ne marchent pas (Linux)

Vous utilisez probablement Wayland. Le premier plan, la position des fenêtres et les raccourcis globaux y dépendent du compositeur, alors qu'ils fonctionnent sous X11. Voir [Installation](./installation.md#x11-et-wayland).

## Il n'y a pas d'icône dans la zone de notification

Certains bureaux Linux, GNOME en particulier, masquent ces icônes sans une extension qui les prend en charge. Le widget et ses boutons continuent de fonctionner. Sur les autres systèmes, regardez dans la zone des icônes cachées de la barre des tâches.

## J'ai lancé Tomatick deux fois

Une seule copie fonctionne à la fois. Relancer Tomatick affiche le widget de la copie déjà ouverte et ne fait rien d'autre.

## J'ai verrouillé l'écran et le minuteur a continué

Tomatick détecte que l'ordinateur s'est mis en veille, mais pas encore que l'écran a été verrouillé. Utilisez le bouton pause, ou laissez l'ordinateur se mettre en veille.

## Pourquoi une phase de travail passée n'est-elle pas comptée ?

Seule une phase de travail menée à son terme est un pomodoro. Une phase passée est enregistrée comme telle, mais elle vous fait quand même avancer dans le cycle, pour que vous arriviez à votre pause longue.

## Puis-je lancer un minuteur et un Pomodoro en même temps ?

Non. Un seul mode est actif à la fois, et changer de mode remet à zéro le minuteur que vous quittez.

## Que se passe-t-il si je quitte pendant une phase ?

La phase est enregistrée comme interrompue au prochain démarrage de l'application. Une phase à moitié terminée n'est pas reprise.

## Comment revenir au cycle par défaut ?

Dans les Réglages, cliquez sur **Revenir au classique**, puis sur **Enregistrer**.

## Comment supprimer Tomatick et ses données ?

Désinstallez-le comme n'importe quelle application, puis supprimez le dossier de données indiqué dans [Installation](./installation.md#où-sont-stockées-vos-données).

## J'ai trouvé un bug ou j'ai une idée

[Ouvrez un ticket](https://github.com/eddiegent/tomatick/issues), s'il vous plaît. Si vous voulez le corriger vous-même, voir [Contribuer](./contributing.md).
