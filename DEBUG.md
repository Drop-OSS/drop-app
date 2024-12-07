# How to create Flamegraph

Run this in `src-tauri`:
```
WEBKIT_DISABLE_DMABUF_RENDERER=1 CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release
```

You can leave out `WEBKIT_DISABLE_DMABUF_RENDERER=1` if you're not on NVIDIA/Linux

And then run this in the root dir:
```
yarn dev --port 1432
```

And then do what you want, and it'll create the flamegraph for you
