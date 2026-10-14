---
title: Contribuer
sidebar_position: 5
---

# Contribuer

Tomatick est un logiciel libre sous [licence MIT](https://github.com/eddiegent/tomatick/blob/main/LICENSE). Les rapports de bugs, les idées, les traductions et le code sont les bienvenus. Pour un changement plus important qu'une petite correction, ouvrez d'abord un ticket pour en discuter.

La conception est décrite dans [SPEC.md](https://github.com/eddiegent/tomatick/blob/main/SPEC.md) (en anglais). Elle explique ce que fait l'application et pourquoi, et ce qui est hors du périmètre (ni liste de tâches, ni comptes, ni cloud). Lisez-la avant de proposer une fonction.

## Ce qu'il vous faut

- [Node.js](https://nodejs.org) 22 ou plus récent
- [Rust](https://rustup.rs) (stable)
- Les [dépendances système de Tauri](https://tauri.app/start/prerequisites/) pour votre système. Sous Linux, cela comprend WebKitGTK 4.1.

## Lancer l'application

```bash
npm install            # installer les dépendances du frontend
npm run tauri dev      # lancer l'application en mode développement
npm run build          # vérifier les types et compiler le frontend
npm run tauri build    # construire les installateurs pour votre système
```

## Tester

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # le cœur en Rust
npm run test:ui                                   # l'interface, avec un faux backend
```

Le moteur (`src-tauri/src/engine`) est du Rust pur, sans dépendance à Tauri, et ses tests sont donc rapides. Les tests de l'interface utilisent Playwright avec un substitut du backend Tauri (`tests/ui/mock.ts`), et s'exécutent dans un simple navigateur.

Les tests de bout en bout pilotent la vraie application avec `tauri-driver` et WebdriverIO. Ils couvrent les commandes, la base de données et les réglages qui survivent à un redémarrage. Voir `tests/e2e/README.md` pour les lancer, sous Linux et Windows. Certaines choses ne peuvent pas être automatisées et restent manuelles : l'icône de la zone de notification, les raccourcis globaux tapés depuis une autre application, la veille, le débranchement d'un écran, le lancement à l'ouverture de session et les notifications système.

L'intégration continue construit l'application sous Windows, macOS et Linux pour chaque pull request, et lance les vérifications. Lancez `cargo fmt` et assurez-vous que `cargo clippy` ne dit rien avant de pousser.

## Documentation

Ce site est dans le dossier `website` et utilise [Docusaurus](https://docusaurus.io).

```bash
cd website
npm install
npm start                  # anglais, avec rechargement automatique
npm start -- --locale fr   # français
npm run build              # construire les deux langues
```

Les pages en anglais sont dans `website/docs`, et celles en français dans `website/i18n/fr/docusaurus-plugin-content-docs/current`. Quand vous modifiez une page, modifiez les deux si vous le pouvez, et dites-le dans votre pull request si vous ne pouvez en faire qu'une.

Les images sont dans `website/static/img/screenshots` (anglais) et `website/static/img/screenshots/fr` (français). Elles sont générées à partir de l'interface avec des données d'exemple par `npm run docs:screenshots`, à lancer depuis la racine du dépôt. Pour en remplacer une par une vraie capture, enregistrez-la sous le même nom dans le même dossier.

## Pull requests

- Une pull request, un sujet.
- Ajoutez ou mettez à jour les tests quand vous changez un comportement.
- Le texte de l'interface passe par les fichiers de traduction. N'écrivez pas de chaînes en dur, voir le [guide de traduction](./translation.md).
- Les changements de conception vont aussi dans SPEC.md.
