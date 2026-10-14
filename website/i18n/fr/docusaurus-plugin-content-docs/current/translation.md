---
title: Guide de traduction
sidebar_position: 7
---

# Guide de traduction

Ajouter une langue à Tomatick demande un fichier JSON et aucun code. L'anglais et le français sont inclus.

## Ajouter une langue

1. Copiez `locales/en.json` vers `locales/xx.json`, où `xx` est le [code ISO 639-1](https://fr.wikipedia.org/wiki/Liste_des_codes_ISO_639-1) de la langue, comme `de` ou `es`.
2. Donnez à `language.name` le nom de la langue **dans cette langue**. C'est ce qu'affiche la liste des langues (`Deutsch`, et non `Allemand`).
3. Traduisez les valeurs. Ne changez pas les clés.
4. Lancez l'application (`npm run tauri dev`), choisissez la langue dans les Réglages, et regardez chaque fenêtre.
5. Ouvrez une pull request.

L'interface trouve le nouveau fichier toute seule, et le côté Rust l'intègre à la compilation, donc le menu de la zone de notification et les notifications sont traduits aussi. La langue est choisie automatiquement quand elle correspond à celle du système.

## Règles

- **Les clés restent identiques.** Seules les valeurs changent.
- **Les paramètres** comme `{{minutes}}` ou `{{version}}` doivent être conservés tels quels, à l'endroit où ils conviennent dans votre phrase.
- **Les pluriels** utilisent des clés qui se terminent par `_one` et `_other`, comme `days_one` et `days_other`. Ajoutez les formes dont votre langue a besoin (`_zero`, `_two`, `_few`, `_many`) en suivant les [règles de pluriel d'i18next](https://www.i18next.com/translation-function/plurals).
- **Une clé absente** se rabat sur l'anglais : une traduction partielle fonctionne, et vous pourrez la compléter plus tard.
- **La concision compte.** Les boutons et le widget ont peu de place, gardez des mots courts.
- **Les dates, heures et nombres** sont formatés selon la langue active. Vous ne les traduisez pas.

## Ce que contient le fichier

| Section | Où elle apparaît |
|---|---|
| `language` | Le nom de la langue |
| `phase` | Les noms des phases |
| `widget` | Les boutons et infobulles du widget |
| `overlay` | La superposition plein écran |
| `settings` | La fenêtre des réglages, messages d'erreur compris |
| `stats` | La fenêtre des statistiques |
| `tray` | Le menu de la zone de notification |
| `notification` | Les notifications système |
| `dialog`, `windowTitle` | Le sélecteur de fichier et les titres de fenêtres |

## Traduire ce site

Les pages en anglais sont dans `website/docs`. Les pages en français sont dans `website/i18n/fr`, et une nouvelle langue suit le même schéma :

```bash
cd website
npm run write-translations -- --locale xx
```

Ajoutez ensuite la langue aux entrées `i18n.locales` et `localeConfigs` de `docusaurus.config.ts`, traduisez les fichiers créés par cette commande, puis les pages dans `i18n/xx/docusaurus-plugin-content-docs/current`.

Les questions sur la formulation sont les bienvenues dans un [ticket](https://github.com/eddiegent/tomatick/issues).
