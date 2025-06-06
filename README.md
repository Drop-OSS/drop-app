# Drop App

Drop app is the companion app for [Drop](https://github.com/Drop-OSS/drop). It uses a Tauri base with Nuxt 3 + TailwindCSS on top of it, so we can re-use components from the web UI.

## Running
Before setting up the drop app, be sure that you have a server set up. 
The instructions for this can be found on the [Drop Docs](https://docs.droposs.org/guides/quickstart.html)

## Current features
Currently supported are the following features:
- Signin (with custom server)
- Database registering & recovery
- Dynamic library fetching from server
- Installing & uninstalling games
- Download progress monitoring
- Launching / playing games

## Development

Install dependencies with `yarn`

Run the app in development with `yarn tauri dev`. NVIDIA users on Linux, use shell script `./nvidia-prop-dev.sh`

To manually specify the logging level, add the environment variable `RUST_LOG=[debug, info, warn, error]` to `yarn tauri dev`:

e.g. `RUST_LOG=debug yarn tauri dev`

## Contributing
Check the original [Drop repo](https://github.com/Drop-OSS/drop/blob/main/CONTRIBUTING.md) for contributing guidelines. 