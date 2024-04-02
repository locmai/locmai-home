/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["*.html", "./src/**/*.rs",],
  darkMode: 'selector',
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
  ],
}
