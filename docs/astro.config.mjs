// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
// import tailwindcss from "@tailwindcss/vite";
import starlightThemeFlexoki from "starlight-theme-flexoki";

export default defineConfig({
  site: "https://stargazers.club",
  integrations: [
    starlight({
      title: "Human 2 Agent",
      plugins: [starlightThemeFlexoki()],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/withastro/starlight",
        },
      ],
      sidebar: [
        {
          label: "Guides",
          items: [
            // Each item here is one entry in the navigation menu.
            { label: "Example Guide", slug: "guides/example" },
          ],
        },
      ],
      customCss: [
        // Relative path to your custom CSS file
        "./src/styles/custom.css",
      ],
    }),
  ],
  vite: {
    // plugins: [tailwindcss(), starlightThemeFlexoki()],
  },
});
