# About this fork

This is a personal fork of [OpenDeck](https://github.com/nekename/OpenDeck). It tracks upstream releases and adds the changes below. It is not affiliated with or supported by the OpenDeck project, so please report problems with these additions here, not upstream.

## What it adds

| Change                                                                                                                                              | Original branch          |
| --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------ |
| An outline size of 0 draws no title outline (canvas ignores a `lineWidth` of 0, so upstream still drew a 1px outline)                               | `fix/text-outline-zero`  |
| A per-state title **Offset** setting to nudge titles up or down                                                                                     | `feat/title-offset`      |
| A font combo box: always lists every font, arrow keys apply fonts live, Escape reverts                                                              | `feat/font-picker`       |
| Key images rendered at each device's native resolution (Stream Decks report theirs; device plugins can declare `keySize` when registering a device) | `feat/device-key-size`   |
| Drag-to-reorder profiles; the profile menu, switching and numbering all use that order                                                              | `feat/profile-order`     |
| Starter pack Switch Profile: dropdowns, and next / previous (optionally within a folder)                                                            | `feat/profile-switching` |
| Starter pack **Page Indicator** action: shows the profile's name, number, or number / total                                                         | `feat/page-indicator`    |

The feature branches are kept for reference; `main` is the fork's product branch.

All additions are backward compatible: profiles and settings written by this fork still load in upstream OpenDeck (the extra fields are ignored), and upstream profiles load here.

## Remotes

- `origin`: this fork
- `upstream`: nekename/OpenDeck (fetch only; pushing is disabled)

## Pulling in upstream changes

Merge upstream **release tags** rather than upstream `main`:

```sh
git fetch upstream --tags
git switch main
git merge v2.15.0   # the new upstream release tag
```

Conflicts, if any, are most likely in `src/lib/rendererHelper.ts`, `src/components/InstanceEditor.svelte`, `src/components/ProfileManager.svelte`, `src-tauri/src/store/profiles.rs` and the starter pack's `src/switch_profile.rs`.

To take a single fix before it is released: `git cherry-pick <commit>` from `upstream/main`.

## Versioning

If the version is changed to mark a fork build, use semver **build metadata** (`2.14.2+fork.1`), not a pre-release suffix (`2.14.2-fork.1`). The startup update check compares against upstream's latest release, and a pre-release counts as older, so it would always report an update.

The update check only shows a notice; it never installs anything. It is a convenient reminder to merge the new upstream tag, and can be turned off in Settings.

## Building on Windows

- Use the MSVC Rust toolchain (`stable-x86_64-pc-windows-msvc`) with the Visual Studio C++ build tools and Windows SDK installed. A GNU toolchain build needs `WebView2Loader.dll` beside the executable and makes OpenDeck look for `x86_64-pc-windows-gnu` plugin binaries, which most plugins do not ship.
- Build from PowerShell or a Developer prompt, not Git Bash: Git Bash's `link` command shadows the MSVC linker.
- Do not set `CARGO_TARGET_DIR` for the whole build. The build script compiles the bundled starter pack with a nested `cargo install`, which inherits it and then waits forever on the outer build's lock. To use a different target directory, set `target-dir` in `src-tauri/.cargo/config.toml` instead.
- `deno task tauri build --no-bundle` builds just the executable; leave out `--no-bundle` for installers.
