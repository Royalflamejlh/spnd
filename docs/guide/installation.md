# Installation

spnd can be installed and used in several ways depending on your preferences and use case.

## Why Direct Execution Works Well

You do not need to install spnd globally before trying it. Direct package runners work well for ad hoc usage:

- ✅ No global package to manage
- ✅ Easy access to the latest published version
- ✅ Cached package downloads after the first run

## Quick Start (Recommended)

The fastest way to use spnd is to run it directly:

::: code-group

```bash [bunx (Recommended)]
bunx @spnd/spnd
```

```bash [pnpm]
pnpm dlx @spnd/spnd
```

```bash [npx]
npx @spnd/spnd@latest
```

```bash [pkg.pr.new preview]
bunx -p https://pkg.pr.new/Royalflamejlh/spnd@<pr-number> spnd --offline
```

:::

::: tip Speed Recommendation
We recommend [bunx](https://bun.com/docs/pm/bunx) for everyday use. It caches the downloaded package, so repeated runs are faster after the first launch.
:::

### Performance Comparison

Here's why runtime choice matters:

| Runtime  | First Run | Subsequent Runs | Notes                        |
| -------- | --------- | --------------- | ---------------------------- |
| bunx     | Fast      | **Instant**     | Recommended for everyday use |
| pnpm dlx | Fast      | Fast            | Good alternative             |
| npx      | Slow      | Moderate        | Widely available             |

## Global Installation (Optional)

You can install spnd globally if you prefer a persistent command:

::: code-group

```bash [npm]
npm install -g @spnd/spnd
```

```bash [bun]
bun install -g @spnd/spnd
```

```bash [yarn]
yarn global add @spnd/spnd
```

```bash [pnpm]
pnpm add -g @spnd/spnd
```

:::

After global installation, run commands directly:

```bash
spnd daily
spnd monthly --breakdown
spnd blocks --live
```

## Native Binaries and Package Managers

Prebuilt binaries skip the JavaScript runtime entirely:

::: code-group

```bash [install script (Linux/macOS)]
curl -fsSL https://raw.githubusercontent.com/Royalflamejlh/spnd/main/install.sh | sh
```

```bash [Homebrew]
brew tap royalflamejlh/spnd https://github.com/Royalflamejlh/spnd
brew install spnd
```

```powershell [Scoop (Windows)]
scoop bucket add spnd https://github.com/Royalflamejlh/spnd
scoop install spnd
```

```bash [Cargo]
cargo install --git https://github.com/Royalflamejlh/spnd spnd
```

```bash [Nix]
nix run github:Royalflamejlh/spnd
```

:::

Tarballs, zips, and `.deb`/`.rpm` packages for every platform are attached to
each [GitHub release](https://github.com/Royalflamejlh/spnd/releases)
(`apt install ./spnd-linux-x64.deb`, `dnf install ./spnd-linux-x64.rpm`).

## Development Installation

For development or contributing to spnd:

```bash
# Clone the repository
git clone https://github.com/Royalflamejlh/spnd.git
cd spnd

# Allow direnv to load the Nix dev shell
direnv allow
```

The Nix dev shell provides the pinned `pnpm`, Rust toolchain, GitHub CLI, git hooks, package tooling, and project utilities. Run project tasks with `just`:

```bash
# Format the tree
just fmt

# Run tests
just test

# Run static checks
just check

# Build distribution
just build
```

You can also run the package directly from source:

```bash
pnpm --filter @spnd/spnd start daily
pnpm --filter @spnd/spnd start monthly --json
```

## Runtime Requirements

### Node.js

- Needed when using Node-based package runners or npm-style global installs
- Use Bun for direct execution when available

### Bun

- **Minimum**: Bun 1.3+
- **Recommended**: Latest stable release
- Recommended for `bunx @spnd/spnd` and for the fastest warm startup

## Verification

After installation, verify spnd is working:

```bash
# Check version
spnd --version

# Run help command
spnd --help

# Test with daily report
spnd daily
```

## Updating

### Direct Execution (npx/bunx)

Always gets the latest version automatically.

### Global Installation

```bash
# Update with npm
npm update -g spnd

# Update with bun
bun update -g spnd
```

### Check Current Version

```bash
spnd --version
```

## Uninstalling

### Global Installation

::: code-group

```bash [npm]
npm uninstall -g @spnd/spnd
```

```bash [bun]
bun remove -g spnd
```

```bash [yarn]
yarn global remove spnd
```

```bash [pnpm]
pnpm remove -g spnd
```

:::

### Development Installation

```bash
# Remove cloned repository
rm -rf spnd/
```

## Troubleshooting Installation

### Permission Errors

If you get permission errors during global installation:

::: code-group

```bash [npm]
# Use npx instead of global install
npx @spnd/spnd@latest

# Or configure npm to use a different directory
npm config set prefix ~/.npm-global
export PATH=~/.npm-global/bin:$PATH
```

```bash [Node Version Managers]
# Use nvm
nvm install 22
npm install -g @spnd/spnd

# Or use fnm
fnm install 22
npm install -g @spnd/spnd
```

:::

### Network Issues

If installation fails due to network issues:

```bash
# Try with different registry
npm install -g @spnd/spnd --registry https://registry.npmjs.org

# Or use bunx for offline-capable runs
bunx @spnd/spnd
```

### Version Conflicts

If you have multiple versions installed:

```bash
# Check which version is being used
which spnd
spnd --version

# Uninstall and reinstall
npm uninstall -g @spnd/spnd
npm install -g @spnd/spnd@latest
```

## Next Steps

After installation, check out:

- [Getting Started Guide](/guide/getting-started) - Your first usage report
- [Configuration](/guide/configuration) - Customize spnd behavior
- [Daily Usage](/guide/daily-reports) - Understand daily usage patterns
