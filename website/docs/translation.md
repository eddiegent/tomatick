---
title: Translation guide
sidebar_position: 7
---

# Translation guide

Adding a language to Tomatick takes one JSON file and no code. English and French are included.

## Add a language

1. Copy `locales/en.json` to `locales/xx.json`, where `xx` is the [ISO 639-1 code](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes) of the language, such as `de` or `es`.
2. Set `language.name` to the name of the language **in that language**. This is what the language list shows (`Deutsch`, not `German`).
3. Translate the values. Do not change the keys.
4. Run the app (`npm run tauri dev`), choose the language in Settings, and look at every window.
5. Open a pull request.

The interface finds the new file by itself, and the Rust side compiles it in, so the tray menu and the notifications are translated too. The language is picked automatically when it matches the language of the system.

## Rules

- **Keys stay the same.** Only values change.
- **Placeholders** such as `{{minutes}}` or `{{version}}` must be kept as they are, wherever they fit in your sentence.
- **Plurals** use keys that end in `_one` and `_other`, such as `days_one` and `days_other`. Add the forms your language needs (`_zero`, `_two`, `_few`, `_many`) following the [i18next plural rules](https://www.i18next.com/translation-function/plurals).
- **A missing key** falls back to English, so a partial translation works, and you can improve it later.
- **Short text** matters. Buttons and the widget have little room, so keep words short.
- **Dates, times and numbers** are formatted with the active language. You do not translate them.

## What is in the file

| Section | Where it appears |
|---|---|
| `language` | The name of the language |
| `phase` | The names of the phases |
| `widget` | The buttons and tooltips of the widget |
| `overlay` | The full-screen overlay |
| `settings` | The settings window, including the error messages |
| `stats` | The statistics window |
| `tray` | The tray menu |
| `notification` | The system notifications |
| `dialog`, `windowTitle` | The file picker and the window titles |

## Translating this site

The English pages are in `website/docs`. The French pages are in `website/i18n/fr`, and a new language follows the same pattern:

```bash
cd website
npm run write-translations -- --locale xx
```

Then add the language to the `i18n.locales` and `localeConfigs` entries of `docusaurus.config.ts`, and translate the files that command created, then the pages in `i18n/xx/docusaurus-plugin-content-docs/current`.

Questions about wording are welcome in an [issue](https://github.com/eddiegent/tomatick/issues).
