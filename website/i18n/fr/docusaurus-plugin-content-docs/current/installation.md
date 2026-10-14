---
title: Installation
sidebar_position: 1
---

# Installation

Les installateurs pour Windows, macOS et Linux sont publiés sur la [page des versions](https://github.com/eddiegent/tomatick/releases). Choisissez le fichier correspondant à votre système :

| Système | Fichier |
|---|---|
| Windows 10 et 11 | installateur `.exe` |
| macOS (Intel et Apple silicon) | `.dmg` |
| Linux | `.AppImage`, ou `.deb` pour Debian et Ubuntu |

:::info Les versions ne sont pas signées
Tomatick est un projet libre, et ses installateurs ne sont pas signés avec un certificat de signature de code payant. Windows et macOS affichent donc un avertissement à la première ouverture. Les étapes ci-dessous expliquent comment continuer. Vous pouvez aussi compiler l'application vous-même à partir du code source, voir [Contribuer](./contributing.md).
:::

## Windows

1. Téléchargez l'installateur `.exe` et lancez-le.
2. Si Windows SmartScreen affiche « Windows a protégé votre ordinateur », cliquez sur **Informations complémentaires**, puis sur **Exécuter quand même**.
3. Windows 11 inclut déjà le composant WebView2 dont Tomatick a besoin. Sous Windows 10, l'installateur le télécharge s'il manque.

## macOS

1. Ouvrez le `.dmg` et glissez Tomatick dans le dossier Applications.
2. La première fois, macOS peut indiquer qu'il ne peut pas vérifier l'application. Ouvrez le dossier **Applications**, maintenez la touche **Contrôle** en cliquant sur Tomatick, choisissez **Ouvrir**, puis confirmez.
3. Si cela n'est pas proposé, ouvrez **Réglages Système**, puis **Confidentialité et sécurité**, descendez dans la page et cliquez sur **Ouvrir quand même** à côté du message sur Tomatick.

## Linux

**AppImage :** rendez le fichier exécutable et lancez-le.

```bash
chmod +x Tomatick_*.AppImage
./Tomatick_*.AppImage
```

**Debian et Ubuntu :** installez le fichier `.deb`. Le paquet dépend de WebKitGTK, que le gestionnaire de paquets installe pour vous.

```bash
sudo apt install ./Tomatick_*.deb
```

### L'icône de la zone de notification sous Linux

L'icône de la zone de notification permet de retrouver l'application quand le widget est masqué. Certains bureaux, GNOME en particulier, n'affichent pas ces icônes sans une extension comme la prise en charge d'AppIndicator. Sans icône, le widget et ses boutons fonctionnent quand même.

### X11 et Wayland

Tomatick fonctionne mieux sous X11. Sous Wayland, c'est le compositeur qui décide de ce qu'une application a le droit de faire, et certaines fonctions sont limitées :

- **Toujours au premier plan :** le widget peut ne pas rester au-dessus des autres fenêtres.
- **Position de la fenêtre :** la position est choisie par le compositeur, donc le widget peut ne pas revenir là où vous l'aviez laissé.
- **Raccourcis globaux :** les raccourcis tapés pendant qu'une autre application a le focus peuvent ne pas être reçus. Le menu de la zone de notification et les boutons du widget restent utilisables.

Les Réglages affichent une erreur à côté d'un raccourci qui n'a pas pu être enregistré, au lieu d'échouer en silence. Si vous avez besoin de ces trois fonctions, ouvrez une session X11.

## Mises à jour

Tomatick demande à GitHub, au démarrage puis une fois par jour, s'il existe une version plus récente. Si c'est le cas, un message apparaît dans le menu de la zone de notification et dans les Réglages, avec un lien vers la page des versions. Il ne télécharge ni n'installe jamais rien tout seul. C'est la seule requête réseau que fait l'application, et vous pouvez la désactiver dans les Réglages, section Système.

## Où sont stockées vos données

Les réglages et l'historique sont réunis dans un seul fichier SQLite, `tomatick.db`, dans le dossier de données d'application de votre système. Les sons d'alerte personnalisés sont copiés dans un dossier `sounds` à côté. Désinstaller l'application ne supprime pas ce dossier : supprimez-le vous-même pour repartir de zéro.

| Système | Emplacement |
|---|---|
| Windows | `%APPDATA%\io.github.eddiegent.tomatick` |
| macOS | `~/Library/Application Support/io.github.eddiegent.tomatick` |
| Linux | `~/.local/share/io.github.eddiegent.tomatick` |
