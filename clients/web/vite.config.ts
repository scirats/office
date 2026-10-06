import path from "node:path"
import { fileURLToPath } from "node:url"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

const dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: [
      { find: "@", replacement: path.resolve(dirname, "./src") },
      { find: /^winbox$/, replacement: path.resolve(dirname, "./node_modules/winbox/src/js/winbox.js") },
    ],
  },
})
