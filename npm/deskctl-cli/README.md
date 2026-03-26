# deskctl-cli

`deskctl-cli` installs the `deskctl` command for Linux X11 systems.

## Install

```bash
npm install -g deskctl-cli
```

After install, run:

```bash
deskctl --help
```

One-shot usage is also supported:

```bash
npx deskctl-cli --help
```

## Runtime Support

- Linux
- X11 session
- currently packaged release asset: `linux-x64`

`deskctl-cli` downloads the matching GitHub Release binary during install.
Unsupported targets fail during install with a clear runtime support error instead of installing a broken command.

If you want the Rust source-install path instead, use:

```bash
cargo install deskctl
```
