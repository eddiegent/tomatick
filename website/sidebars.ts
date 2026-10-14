import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  docs: [
    "installation",
    "user-guide",
    "configuration",
    "faq",
    {
      type: "category",
      label: "For contributors",
      collapsed: false,
      items: ["contributing", "architecture", "translation"],
    },
  ],
};

export default sidebars;
