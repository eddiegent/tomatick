import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Tomatick",
  tagline: "A free, open source Pomodoro timer with an always-on-top tomato widget",
  favicon: "img/favicon.svg",

  // The site is served by GitHub Pages, as a project site of the repository.
  url: "https://eddiegent.github.io",
  baseUrl: "/tomatick/",
  organizationName: "eddiegent",
  projectName: "tomatick",
  trailingSlash: false,

  onBrokenLinks: "throw",
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: "throw",
    },
  },

  // English and French. The French pages live in i18n/fr.
  i18n: {
    defaultLocale: "en",
    locales: ["en", "fr"],
    localeConfigs: {
      en: { label: "English", htmlLang: "en" },
      fr: { label: "Français", htmlLang: "fr" },
    },
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          editUrl: "https://github.com/eddiegent/tomatick/tree/main/website/",
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: "Tomatick",
      logo: {
        alt: "Tomatick",
        src: "img/logo.svg",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docs",
          position: "left",
          label: "Documentation",
        },
        {
          type: "localeDropdown",
          position: "right",
        },
        {
          href: "https://github.com/eddiegent/tomatick",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Documentation",
          items: [
            { label: "Installation", to: "/docs/installation" },
            { label: "User guide", to: "/docs/user-guide" },
            { label: "FAQ", to: "/docs/faq" },
          ],
        },
        {
          title: "Project",
          items: [
            { label: "Source code", href: "https://github.com/eddiegent/tomatick" },
            { label: "Releases", href: "https://github.com/eddiegent/tomatick/releases" },
            { label: "Report a problem", href: "https://github.com/eddiegent/tomatick/issues" },
          ],
        },
      ],
      copyright: "Copyright © 2026 Eddie Gent. Tomatick is free software under the MIT license.",
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["bash", "rust", "json"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
