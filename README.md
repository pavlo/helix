<div align="center">

<h1>
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
  <img alt="Helix" height="128" src="logo_light.svg">
</picture>
</h1>

**A personal fork of [Helix](https://github.com/helix-editor/helix)** — a post-modern modal text editor written in Rust.

[![Upstream](https://img.shields.io/badge/upstream-helix--editor%2Fhelix-452859?logo=github)](https://github.com/helix-editor/helix)
[![Branch](https://img.shields.io/badge/branch-my--helix-blue?logo=git)](https://github.com/pavlo/helix/tree/my-helix)
[![License](https://img.shields.io/badge/license-MPL--2.0-green)](./LICENSE)

</div>

> **This is not the official Helix repository.**
> For releases, documentation and community support, use
> [helix-editor/helix](https://github.com/helix-editor/helix) and
> [docs.helix-editor.com](https://docs.helix-editor.com/).

# What is this?

This repository is my personal fork of Helix. It tracks upstream `master` and adds a
small set of patches that I use every day: features from upstream pull requests that
have not been merged yet, plus a few local additions.

The fork's integration branch is **`my-helix`**. Build that branch if you want the
changes described below.

- Upstream: <https://github.com/helix-editor/helix>
- This fork: <https://github.com/pavlo/helix>
- Upstream base at the time of writing: `ba40e5474`

# Why this fork?

Helix upstream has a large backlog and limited review/merge capacity. Some fixes and
features that matter for my workflow have been open for a long time, and waiting for
them is not practical for an editor I use daily. This fork exists to:

- keep the upstream features I depend on in a single, buildable branch;
- import selected upstream pull requests without waiting for them to be merged;
- add small local patches (for example, language support) that are useful to me;
- keep the changes isolated, so they can be dropped once upstream catches up.

The goal is not to compete with upstream or to maintain a separate editor. It is a
small, personal patch set on top of Helix.

# What's included

| Change | Source | Upstream status | Import branch / commits |
| --- | --- | --- | --- |
| Automatic reload of files changed externally | [PR #13963](https://github.com/helix-editor/helix/pull/13963) by [@AnthonyMichaelTDM](https://github.com/AnthonyMichaelTDM) | Closed, not merged | `pr-13963` |
| Recent files picker | [PR #15022](https://github.com/helix-editor/helix/pull/15022) by [@ascyii](https://github.com/ascyii) | Open | `pr-15022` |
| Multi-root file picker | Local addition | — | `bf85c7084`, `2f4f0aa8c` |
| MQL5 language support | Local addition | — | `d386a9117`, `57bef9bec` |

The upstream pull requests are merged into `my-helix` as separate merge commits. This
keeps the provenance of each change visible and makes it easy to remove a patch later.

## 1. Automatic reload when files change externally

Imported from [PR #13963](https://github.com/helix-editor/helix/pull/13963).

Helix detects documents that were modified outside the editor and offers to reload
them. Detection is based on file modification times and the existing event system, so
it works on any platform that reports file modification times and does not pull in an
extra file-watching runtime.

The feature is **disabled by default**. Enable it in `config.toml`:

```toml
# Reload when the terminal gains focus (requires terminal focus-event support)
[editor]
auto-reload = true
```

or configure it in detail:

```toml
[editor.auto-reload]
# Check for external changes when Helix gains focus.
focus-gained = true

# Also check periodically.
periodic.enable = true
periodic.interval = 5000 # milliseconds
```

| Key | Description | Default |
| --- | --- | --- |
| `focus-gained` | Check for externally modified files when Helix gains focus | `false` |
| `periodic.enable` | Check for externally modified files on a timer | `false` |
| `periodic.interval` | Timer interval in milliseconds | `3000` |

Notes:

- Files with unsaved changes are not reloaded automatically.
- Reloading is postponed while you are in insert mode, then requested when you leave it.
- `focus-gained` requires a terminal that reports focus events. See the
  [terminal support table](https://github.com/helix-editor/helix/wiki/Terminal-Support).

## 2. Recent files picker

Imported from [PR #15022](https://github.com/helix-editor/helix/pull/15022).

A picker for recently saved files:

- press `Space l` in normal mode, or run `:recent-picker`;
- paths are recorded when a file is saved;
- duplicate entries are removed and the newest entry is placed first;
- only files that still exist are shown.

The history is stored in Helix's cache directory, for example
`~/.cache/helix/recents.txt` on Linux and macOS.

Configuration:

```toml
[editor.recent-picker]
# Maximum number of entries shown in the picker.
max-show-entries = 50

# Maximum number of entries kept in the recents file.
max-save-entries = 50

# Optional custom location for the recents file.
# recents-path = "/path/to/recents.txt"
```

| Key | Description | Default |
| --- | --- | --- |
| `max-show-entries` | Maximum number of entries shown in the picker | `50` |
| `max-save-entries` | Maximum number of entries written to the recents file | `50` |
| `recents-path` | Path to the recents file | OS cache directory + `helix/recents.txt` |

Notes:

- Entries are currently added when a file is **saved**, not when it is merely opened.
- There is no option to disable recording yet; lower `max-save-entries` if needed.

## 3. Multi-root file picker

Local addition, not an upstream PR.

A second file picker that searches files across several configured directories in a
single picker. Open it with `Space m` in normal mode.

```toml
[editor.file-picker]
search-paths = [
    "~/Projects",
    "~/Documents",
    "~/Work",
    "~/Notes",
]
```

| Key | Description |
| --- | --- |
| `Space m` | Open the file picker across all configured search paths |

Notes:

- Paths support `~` expansion.
- Unavailable directories are ignored; the remaining valid paths are still searched.
- An empty or missing `search-paths` list does not affect the standard `Space f` picker.
- Results are shown with their full path (home folded to `~`), for example
  `~/Projects/Helix/src/main.rs`, so files with identical relative paths in different
  roots remain distinguishable.
- Matching is performed by the existing `nucleo` implementation over the complete
  displayed path. Root names, nested directories and filenames can all be searched,
  including native multi-term queries.
- The same ignore rules and file-picker options as the standard picker are respected.
- Selecting a result opens its original absolute path.
- Overlapping search roots are deduplicated.

The implementation lives in `helix-term/src/ui/multi_root_picker.rs` and deliberately
leaves the standard `ui::file_picker()` unchanged, keeping upstream merges simple.

## 4. MQL5 language support

Local addition, not an upstream PR.

- File types: `.mq5`, `.mqh`
- Tree-sitter grammar: C++ (`grammar = "cpp"`)
- Queries: inherit from the C++ queries
- Language server: none configured by default

Files:

```text
runtime/queries/mql5/
├── highlights.scm
├── indents.scm
├── injections.scm
├── locals.scm
├── tags.scm
└── textobjects.scm
```

Because MQL5 is based on MQL5/C++ syntax, the C++ grammar provides useful highlighting
and text objects without maintaining a separate grammar. Configure an MQL5 language
server yourself if you need LSP features.

# Building from source

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) — the exact toolchain is pinned in
  [`rust-toolchain.toml`](./rust-toolchain.toml) (`1.90.0`); `rustup` installs it
  automatically;
- Git;
- a C++14-compatible compiler for the tree-sitter grammars (Xcode Command Line Tools
  on macOS, GCC or Clang on Linux);
- optionally [Nix](https://nixos.org/) with flakes, or `direnv`.

## Clone and run

```sh
git clone git@github.com:pavlo/helix.git
cd helix
git switch my-helix
```

Build and run a debug binary:

```sh
cargo run
cargo run -- path/to/file
```

Debug builds compile much faster and are usually good enough for development.

Build a release binary:

```sh
cargo build --release
./target/release/hx
```

Or use the repository's optimized `opt` profile (fat LTO, one codegen unit, stripped):

```sh
cargo build --profile opt --locked
./target/opt/hx
```

The binary can also be installed into `~/.cargo/bin`:

```sh
# Reproducible release build
cargo install --path helix-term --locked

# Or an optimized build for the current machine
cargo install \
  --profile opt \
  --config 'build.rustflags=["-C", "target-cpu=native"]' \
  --path helix-term \
  --locked
```

## Runtime files

When running from the repository, Helix uses the sibling `runtime/` directory. For an
installed binary, point Helix at the runtime directory:

```sh
# Linux/macOS: symlink the runtime directory
mkdir -p ~/.config/helix
ln -sfn "$PWD/runtime" ~/.config/helix/runtime

# Alternative: set an environment variable
export HELIX_RUNTIME="$PWD/runtime"
```

Check the installation with:

```sh
hx --health
```

If you prefer to fetch and compile grammars explicitly:

```sh
hx --grammar fetch
hx --grammar build
```

Set `HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1` if you do not want grammars to be fetched or
built during compilation.

## Nix

A flake and a legacy `shell.nix` are included. With flakes and `direnv`:

```sh
direnv allow
cargo run
```

Or use the development shell directly:

```sh
nix develop
cargo run
```

# Development workflow

Formatting, linting and tests:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

Integration tests for `helix-term`:

```sh
cargo integration-test
```

Useful development commands:

```sh
# Regenerate the command/language documentation in book/
cargo xtask docgen

# Preview the documentation book
mdbook serve book
```

For logging, run `hx -v` (more `v`s increase verbosity), use `:log-open`, or start with
`cargo run -- --log foo.log` and follow the log with `tail -f foo.log`. The
`HELIX_LOG_LEVEL` environment variable also works, for example
`HELIX_LOG_LEVEL=debug cargo integration-test`.

See [`docs/CONTRIBUTING.md`](./docs/CONTRIBUTING.md) for upstream development
conventions.

# Keeping the fork in sync

The integration branch is `my-helix`. Because imported pull requests use merge
commits, merge upstream into the fork instead of rebasing public history:

```sh
git fetch upstream
git switch my-helix
git merge upstream/master

# Resolve conflicts, then verify the build
cargo build --profile opt --locked
cargo test --workspace

git push origin my-helix
```

To import another upstream pull request:

```sh
# Fetch the PR head into a local branch
git fetch upstream pull/12345/head:pr-12345

# Merge it into the integration branch, preserving the PR history
git switch my-helix
git merge --no-ff pr-12345
```

If upstream eventually merges one of these features, drop the corresponding import
merge commit (or resolve the conflict in favor of upstream) and keep the local patch
set small.

# Limitations

- This is a personal fork: there are no releases or pre-built binaries, and no
  guarantee of stability or support.
- Do not report bugs caused by these fork-only changes to upstream Helix. Check
  whether the issue also occurs on upstream `master` first.
- Fork features may conflict with upstream changes during a sync; conflicts are
  resolved on the `my-helix` branch.
- The MQL5 integration provides syntax support only; it does not configure a language
  server.
- The recent files picker records files on save rather than on open.

# Credits

Helix is created and maintained by the [Helix contributors](https://github.com/helix-editor/helix/graphs/contributors).
Thanks to [@jakenvac](https://github.com/jakenvac) for the logo, and to
[@AnthonyMichaelTDM](https://github.com/AnthonyMichaelTDM) and
[@ascyii](https://github.com/ascyii) for the upstream pull requests included here.

# License

Helix is licensed under the [Mozilla Public License 2.0](./LICENSE). This fork keeps
the same license.
