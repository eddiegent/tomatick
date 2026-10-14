# Documentation images

These images are **rendered**, not captured. `tests/screenshots/render.spec.ts` draws the real interface in a browser with a fake backend and sample data, so they show the app but not a real desktop: no window frame, no desktop behind the widget, and the fonts of the system are missing.

They are rendered once per language: the English images are in this folder, and the French ones, with the app set to French, are in `fr/` under the same file names. The English pages use the first set and the French pages the second.

They are meant to be replaced. To use a real capture, save it under the same file name in the right folder (`widget-work.png`, `fr/settings.png`, and so on). The pages refer to the images by name, so nothing else changes. A new language `xx` needs an `xx/` folder, and an entry in the `LANGUAGES` list of the render spec.

The widget images are transparent PNGs, shown at 200 px. The other images were taken at twice their size.

## Render them again

From the root of the repository:

```
npm install
npx playwright install chromium
npm run docs:screenshots
```

This overwrites the images in this folder and in `fr/`, including any real capture you put there, so move those aside or restore them with git afterwards.

The same ten files exist in `fr/`.

| File | What it shows |
|---|---|
| `widget-work.png` | The widget during a work phase |
| `widget-controls.png` | The widget with its buttons (hovered) |
| `widget-paused.png` | A paused timer |
| `widget-short-break.png` | A short break, with an exercise |
| `widget-long-break.png` | A long break, part 2 of 3 |
| `widget-countdown.png` | The countdown (amber) |
| `settings.png` | The top of the settings window |
| `settings-alerts.png` | The alerts section of the settings |
| `statistics.png` | The statistics window |
| `overlay-break.png` | The break overlay |
