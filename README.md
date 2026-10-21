# Anarlog Local

An unofficial, independent local-first desktop edition of the MIT-licensed Anarlog community application. It keeps recording, local storage, editing, export, local/BYOK transcription, chat, summaries, playback controls, dictionary, dictation, templates, and local automations available without an Anarlog account or subscription.

Not affiliated with or supported by Fastrepl, Inc. “Anarlog” is used only to identify the upstream project.

## Download and install on Apple Silicon

CI builds a real `aarch64-apple-darwin` `.app` and `.dmg` on GitHub's standard `macos-26` ARM64 runner. Builds are ad-hoc signed and are not Apple-notarized.

1. Download the DMG and `SHA256SUMS.txt` from the GitHub Actions artifact attached to a green commit or from a project prerelease.
2. Verify it: `shasum -a 256 -c SHA256SUMS.txt`.
3. Open the DMG and drag Anarlog Local to Applications.
4. If macOS blocks the first launch, open System Settings → Privacy & Security and choose Open Anyway for Anarlog Local. Do not disable Gatekeeper globally.
5. Grant only the microphone, system-audio, and Accessibility permissions needed by the features you use.

Anarlog Local uses bundle identifier `com.evnluo.anarlog-local`, so its app-support directory and database do not overwrite official Anarlog/Hyprnote installs.

## Local and BYOK setup

Open Settings:

- Transcription: choose an on-device Soniqo/Apple Speech model, a local whisper.cpp model, or configure a supported provider with your own key.
- Intelligence: choose Apple Foundation Models, Ollama/LM Studio, or a supported API provider with your own endpoint/key.
- Dictionary: add names and terminology used by transcription and dictation.
- Dictation: enable the shortcut after granting microphone and Accessibility access.
- Templates and Automations: customize local summary formats and local export workflows.

Provider keys are stored through the application's protected settings/keychain path. Do not commit `.env` files or keys.

## Privacy and network boundary

Core recording, editing, local storage, export, playback, dictionary, and on-device model flows need no Anarlog account. The desktop catalogue hides Anarlog-managed AI, and the webview blocks Anarlog, Hyprnote, Char, and Fastrepl service domains before `fetch` reaches the network.

Network access remains possible only when the user chooses it:

- explicit BYOK AI/STT providers;
- local endpoints such as Ollama or LM Studio;
- public model downloads disclosed by the selected local-model UI;
- ordinary links the user deliberately opens.

Product analytics, crash-reporting credentials, vendor update feeds, billing, cloud sync, cloud sharing, teams, referrals, CRM, managed web search, and managed Anarlog AI are disabled in this edition. See [CAPABILITIES.md](CAPABILITIES.md).

## Build from source

Requirements: macOS 15+, Xcode 26 with the Metal Toolchain component, Node 22, pnpm 11.1.1, and Rust 1.96.0.

```text
pnpm install --frozen-lockfile
pnpm --filter @anlg/ui build
pnpm --filter @anlg/desktop typecheck
pnpm --filter @anlg/desktop test
swiftc plugins/permissions/swift/check-permissions.swift \
  -o apps/desktop/src-tauri/binaries/check-permissions-aarch64-apple-darwin
pnpm --filter @anlg/desktop tauri build \
  --target aarch64-apple-darwin --bundles app,dmg
```

The reproducible CI recipe is `.github/workflows/macos-arm64.yml`. It uses a standard free public-repository `macos-15` ARM64 runner, no secrets, no Apple developer identity, and no upstream signing or updater keys.

## Testing status

Automated checks cover signed-out/free local dictation lifecycle and cancellation, provider-catalogue routing, local capability policy, vendor-domain blocking, independent bundle identity, dictionary behavior, and local automation behavior. CI also validates the native binary architecture, bundle identifier, ad-hoc signature, and process startup.

A real Mac is still required to validate microphone capture quality, system-audio capture, Accessibility insertion into third-party apps, local model performance, and every TCC prompt. Those checks are not claimed by CI.

## License and provenance

Community source is MIT-licensed by Fastrepl, Inc.; see [LICENSE](LICENSE), [LICENSING.md](LICENSING.md), [UPSTREAM.md](UPSTREAM.md), and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). No `enterprise/**` source or upstream release/signing automation is included.
