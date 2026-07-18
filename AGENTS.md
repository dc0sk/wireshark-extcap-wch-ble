# Agents

## Documentation conventions

Always apply the `living-docs` and `doc-frontmatter` skills when creating or editing
documentation files. Specifically:

1. Every doc must open with YAML frontmatter (`project`, `doc`, `status`, `last_updated`).
   The `doc:` field must equal the file's own repo-relative path.
2. Use the living-docs directory layout:
   - `docs/` — user-facing docs
   - `docs/dev/` — developer docs (with `README.md` index)
   - `docs/dev/project/` — tracking docs (changelog, roadmap, etc.)
   - `docs/dev/project.md` — roof index over tracking docs
   - `docs/dev/design/` — architecture & design
   - `docs/dev/reviews/` — point-in-time review snapshots
   - `docs/dev/archive/` — frozen/superseded docs
3. Changelogs go in `docs/dev/project/CHANGELOG.md` and follow [Keep a Changelog](https://keepachangelog.com/).
4. When moving or renaming a doc, update the `doc:` frontmatter and sweep all path references.

## Build & quality

- Build: `cargo build --release`
- The extcap binary must be installed to `/usr/lib/wireshark/extcap/` for Wireshark to discover it.
