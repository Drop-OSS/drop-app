# Drop Desktop Client

The Drop Desktop Client is the companion app for [Drop](https://github.com/Drop-OSS/drop). It is the official & intended way to download and play games on your Drop server.

## Internals

It uses a Tauri base with Nuxt 3 + TailwindCSS on top of it, so we can re-use components from the web UI.

## Development
Before setting up a development environemnt, be sure that you have a server set up. The instructions for this can be found on the [Drop Docs](https://docs.droposs.org/docs/guides/quickstart).

Then, install dependencies with `yarn`. This'll install the custom builder's dependencies. Then, check everything works properly with `yarn tauri build`. 

Run the app in development with `yarn tauri dev`. NVIDIA users on Linux, use shell script `./nvidia-prop-dev.sh`

To manually specify the logging level, add the environment variable `RUST_LOG=[debug, info, warn, error]` to `yarn tauri dev`:

e.g. `RUST_LOG=debug yarn tauri dev`

## Contributing
Check out the contributing guide on our Developer Docs: [Drop Developer Docs - Contributing](https://developer.droposs.org/contributing).
