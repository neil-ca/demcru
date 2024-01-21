/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./templates/**/*.hbs"],
  theme: {
    extend: {},
    fontFamily: {
      space: ['"Space Mono"', "monospace"],
      bree: ['"Bree Serif"', "serif"],
    },
  },
  daisyui: {
    themes: ["dim"],
  },
  plugins: [require("daisyui")],
};
