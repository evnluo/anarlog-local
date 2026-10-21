# Third-party notices

Anarlog Local includes the MIT-licensed Anarlog community source. The full Fastrepl MIT notice is in `LICENSE` and applies to substantial portions of this distribution.

Dependency versions and their upstream package identities are locked in `Cargo.lock` and `pnpm-lock.yaml`. Their licenses remain those declared by the respective packages. Source-distributed notices that require preservation are kept beside the code/assets, including:

- `crates/tiptap/vendor/mdast_util_to_markdown/LICENSE` — vendored mdast-to-markdown license notice;
- `apps/desktop/src/settings/stats/tremor/LICENSE` — Tremor-derived UI notice;
- `plugins/windows/swift-lib/src/Resources/OFL.txt` — SIL Open Font License for Cabin Sketch;
- `licenses/Pretendard-LICENSE.txt` — SIL Open Font License 1.1 for the bundled Pretendard fonts;
- `licenses/sqlite-sync-Apache-2.0.txt` — Apache License 2.0 for the SQLite Sync 1.2.0 library bundled on macOS;
- copyright/license headers in vendored native and generated sources.

Public model files downloaded after installation are not relicensed by this project. Their source, model card, and license are controlled by the selected model host and are shown or linked by the model selection flow where available.

No code from upstream `enterprise/**` is included.
