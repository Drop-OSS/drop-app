// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-04-03",

  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },

  ssr: false,

  extends: ["../shared", "../libs/drop-base"],

  app: {
    baseURL: "/tvmode",
  },

  devtools: {
    enabled: false,
  },

  
});
