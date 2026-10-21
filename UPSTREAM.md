# Upstream provenance

Anarlog Local is based on the MIT community layer of:

- Project: https://github.com/fastrepl/anarlog
- Audited/imported revision: `be8bce795097a3ae5470525965b1a140e9afb7b6`
- Upstream copyright: Copyright (c) 2023-present Fastrepl, Inc.
- License: MIT outside paths controlled by a nearer license; see `LICENSE` and `LICENSING.md`.

The source was imported as a clean snapshot without upstream Git history. This repository excludes:

- `enterprise/**` and `LICENSE.enterprise` (commercially licensed);
- upstream deployment, release, signing, notarization, secret-management, and vendor CI automation;
- server, web, mobile, and CLI applications not needed by the desktop-only edition;
- Supabase deployment state, hosted documentation, agent configuration, and local developer configuration;
- user data, recordings, notes, credentials, environment files, and audit working material.

Local-edition modifications include independent product/bundle identity, account-free local capability policy, managed-service UI removal, provider catalogue filtering, vendor network blocking, disabled telemetry/updater credentials, and a secret-free Apple Silicon build workflow.
