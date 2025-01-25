export default defineNuxtPlugin((nuxtApp) => {
  // Also possible
  nuxtApp.hook("vue:error", (error, instance, info) => {
    console.error(error, info);
    const router = useRouter();
    router.replace(`/error`);
  });
});
