# Drop App

Drop app is the companion app for [Drop](https://github.com/Drop-OSS/drop). It uses a Tauri base with Nuxt 3 + TailwindCSS on top of it, so we can re-use components from the web UI.

## Development

Install dependencies with `yarn`

Run the app in development with `yarn tauri dev`. NVIDIA users on Linux, use the environment variable in `.env`

To manually specify the logging level, add the environment variable `RUST_LOG=[debug, info, warn, error]` to `yarn tauri dev`:

e.g. `RUST_LOG=debug yarn taudi dev`

## Contributing
Check the original Drop repo for contributing guidelines. 