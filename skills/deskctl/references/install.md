# Install `deskctl`

`deskctl` is designed to be used non-interactively by agents. The easiest install path is the npm package because it installs the `deskctl` command directly from GitHub Release assets without needing Cargo on the target machine.

## Preferred: npm global install

```bash
npm install -g deskctl-cli
deskctl --help
```

This is the preferred path for sandboxes, VMs, and sandbox-agent sessions where Node/npm already exists.

## User-prefix npm install

If global npm installs are not writable:

```bash
mkdir -p "$HOME/.local/bin"
npm install -g --prefix "$HOME/.local" deskctl-cli
export PATH="$HOME/.local/bin:$PATH"
deskctl --help
```

This avoids `sudo` and keeps the install inside the user home directory.

## One-shot npm execution

```bash
npx deskctl-cli --help
```

Use this for quick testing. For repeated desktop control, install the command once so the runtime is predictable.

## Fallback: Cargo

```bash
cargo install deskctl
```

Use this only when the machine already has a Rust toolchain or when you explicitly want a source build.

## Fallback: local Docker build

If you need a Linux binary from macOS or another non-Linux host:

```bash
docker compose -f docker/docker-compose.yml run --rm build
```

Then copy `dist/deskctl-linux-x86_64` into the target machine.

## Runtime prerequisites

`deskctl` needs:

- Linux
- X11
- a valid `DISPLAY`
- a working desktop/window-manager session

Quick verification:

```bash
printenv DISPLAY
printenv XDG_SESSION_TYPE
deskctl doctor
```

Inside sandbox-agent, you may need to install desktop dependencies first:

```bash
sandbox-agent install desktop --yes
deskctl doctor
```
