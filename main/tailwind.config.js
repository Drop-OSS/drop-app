/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./components/**/*.{js,vue,ts}",
    "./layouts/**/*.vue",
    "./pages/**/*.vue",
    "./plugins/**/*.{js,ts}",
    "./app.vue",
    "./error.vue",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter"],
        display: ["Motiva Sans"],
      },
    },
  },
  plugins: [require("@tailwindcss/forms"), require('@tailwindcss/typography')],
};
