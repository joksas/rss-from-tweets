/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/tmpl.rs"],
  theme: {
    extend: {},
  },
  plugins: [require("@tailwindcss/typography")],
};
