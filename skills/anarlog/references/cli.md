# CLI commands

Use the `anarlog-local` command installed by Anarlog Local and request JSON for agent-readable output.

```bash
anarlog-local --json doctor
anarlog-local --json meetings --source local list --query "planning" --limit 20 --offset 0
anarlog-local --json meetings --source local folders --query "clients"
anarlog-local --json meetings --source local get MEETING_ID
anarlog-local --json meetings --source local note MEETING_ID --kind note
anarlog-local --json meetings --source local note MEETING_ID --kind summary
anarlog-local --json meetings --source local transcript MEETING_ID --limit 200 --offset 0
anarlog-local --json meetings --source local history MEETING_ID --limit 20 --offset 0
anarlog-local --json proposals list --meeting MEETING_ID
```

Meeting commands stay on `--source local`; Anarlog Local has no hosted fallback. Continue from `pagination.next_offset` only when more transcript context is needed.

For an explicit request to save a complete meeting:

```bash
anarlog-local meetings export MEETING_ID --format markdown --output meeting.md
anarlog-local meetings export MEETING_ID --format json --output meeting.json
```

Export refuses to replace an existing file. Use `--force` only after the user approves overwriting that exact path.

For an explicitly requested audio import, use `anarlog-local --json meetings --source local import FILE --request-key KEY`. The desktop app must be open. Configured local or BYOK providers may process the audio; save the returned job ID and check import status before claiming completion.
