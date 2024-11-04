export default defineNuxtPlugin((nuxtApp) => {
  // Also possible
  nuxtApp.hook("vue:error", (error, instance, info) => {
    console.log(error);
    const router = useRouter();
    router.replace(`/error`);
  });
});
