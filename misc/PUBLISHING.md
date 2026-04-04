# Publishing Guide

## Version Bump

All 8 version-bearing files must stay in sync. Use the bump script:

```bash
# Set an explicit version
./misc/bump-version.sh 0.3.0

# Auto-increment patch (reads Cargo.toml, bumps Z+1)
./misc/bump-version.sh
```

**Files patched:**

| File | Fields |
|------|--------|
| `Cargo.toml` | `version` |
| `npm/weoline/package.json` | `version` + 6 `optionalDependencies` |
| `npm/weoline-darwin-arm64/package.json` | `version` |
| `npm/weoline-darwin-x64/package.json` | `version` |
| `npm/weoline-linux-arm64/package.json` | `version` |
| `npm/weoline-linux-x64/package.json` | `version` |
| `npm/weoline-win32-arm64/package.json` | `version` |
| `npm/weoline-win32-x64/package.json` | `version` |

## Cutting a Release

Releases are fully automated via GitHub Actions. **Never publish packages manually.**

```bash
# 1. Bump version
./misc/bump-version.sh 0.3.0

# 2. Commit
git add -A && git commit -m "Bump version to 0.3.0"

# 3. Tag and push
git tag v0.3.0
git push && git push --tags
```

The `v*` tag triggers the Release workflow (`.github/workflows/release.yml`), which:

1. Compiles platform-specific binaries (macOS ARM/x64, Linux ARM/x64, Windows ARM/x64)
2. Runs unit tests and smoke tests on native-arch targets
3. Attaches archives to a GitHub Release (powers curl/Homebrew installs)
4. Publishes platform npm packages (`@overweo/weoline-*`) — **only after ALL builds pass**
5. Publishes the main `weoline` npm package (depends on all platform packages)

## Trusted Publishers (npm OIDC)

npm packages are published via GitHub Actions OIDC — no `NPM_TOKEN` secret is needed.

### First-time setup (per package)

Link each package to the GitHub repository as a trusted publisher on npmjs.com:

1. Go to **npmjs.com** -> package settings -> **Publishing access**
2. Add a trusted publisher:
   - **Repository:** `OverWeo/weoline`
   - **Workflow:** `release.yml`
   - **Environment:** *(leave blank)*

Repeat for all 7 packages:

```
weoline
@overweo/weoline-darwin-arm64
@overweo/weoline-darwin-x64
@overweo/weoline-linux-arm64
@overweo/weoline-linux-x64
@overweo/weoline-win32-arm64
@overweo/weoline-win32-x64
```

### Creating new scoped packages

New `@overweo/*` packages must be manually published once before OIDC can take over:

```bash
# Log in to npm
npm login

# Initial publish (creates the package on the registry)
cd npm/<new-package>
npm publish --access public

# Then set up trusted publisher on npmjs.com (see above)
```

### How OIDC works in the workflow

The release workflow requests an OIDC token from GitHub (`id-token: write` permission) and exchanges it with npm for a short-lived publish token. This is configured in `release.yml` via:

```yaml
permissions:
  id-token: write
```

and `setup-node` with `registry-url: 'https://registry.npmjs.org'`. No long-lived npm tokens are stored in repository secrets.
