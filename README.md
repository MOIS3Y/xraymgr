# xraymgr

<p align="left">
  <img src="https://img.shields.io/badge/Rust-eba0ac?style=for-the-badge&logo=rust&logoColor=white&labelColor=1e1e2e&label=Rust&messageColor=11111b&color=eba0ac" alt="Rust">
  <img src="https://img.shields.io/badge/Nix-8caaee?style=for-the-badge&logo=nixos&logoColor=white&labelColor=1e1e2e&label=Nix&messageColor=11111b&color=8caaee" alt="Nix">
  <img src="https://img.shields.io/badge/License-MIT-a6e3a1?style=for-the-badge&labelColor=1e1e2e&messageColor=11111b&color=a6e3a1" alt="MIT">
</p>

I wrote this utility to make managing my own Xray server easier. It's a simple, fast tool for adding/removing clients and generating VLESS links with QR codes directly in the terminal.

I understand that it probably doesn't cover all possible Xray configurations (it's strictly focused on VLESS + Reality + TCP right now). I just needed a quick and convenient instrument for managing users and sharing configs on my personal server. If you use it and find it helpful, but it's missing a feature you need, don't hesitate to open a PR!

---

## Installation & Usage

### Running via Nix
If you use Nix, you can run the utility directly without installing it. 
*(Make sure to pass the path to your config and the server address if they are not in the current directory)*:

```bash
nix run github:MOIS3Y/xraymgr -- --config /path/to/config.json list
nix run github:MOIS3Y/xraymgr -- --config /path/to/config.json show "user@example.com" --address "my.vpn.server"
```

### Standard Installation
```bash
git clone https://github.com/MOIS3Y/xraymgr
cd xraymgr
cargo install --path .
```

## Commands

### Listing Clients
```bash
xraymgr list
```

### Adding a Client
```bash
xraymgr add "user@example.com"
```

### Showing Connection Info
```bash
xraymgr show "user@example.com" --address "my.server.com"
```

> [!NOTE]
> You must provide the server address either via the `--address` flag or the `XRAYMGR_ADDRESS` environment variable.

## Configuration & Fallbacks

To avoid typing the same flags over and over, `xraymgr` follows this hierarchy:

| Parameter | CLI Flag | Env Variable | Default / Fallback |
|-----------|----------|--------------|--------------------|
| **Config**| `--config` | `XRAYMGR_CONFIG` | `config.json` |
| **Tag** | `--tag` | `XRAYMGR_TAG` | `vless-in` |
| **Address**| `--address`| `XRAYMGR_ADDRESS`| *None (Required)* |

For example, you can add this to your `.bashrc` or `.envrc`:
```bash
export XRAYMGR_CONFIG="/etc/xray/config.json"
export XRAYMGR_ADDRESS="my.vpn.server"
export XRAYMGR_TAG="vless-reality"
```
And then simply run:
```bash
xraymgr show "user@example.com"
```

## Current Limitations

- **Protocol**: VLESS only.
- **Security**: Reality only.
- **Transport**: TCP (including XTLS Vision). WebSocket, gRPC, etc., are not supported yet.

If your setup is different and you want to add support for it, PRs are welcome!
