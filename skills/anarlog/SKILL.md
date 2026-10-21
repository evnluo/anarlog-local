---
name: anarlog-local
description: Read and work with local Anarlog Local meetings through the installed CLI or local MCP server. Use when a user asks about meetings stored on this computer.
---

# Anarlog Local

Use only the local Anarlog Local database through the supported CLI or local MCP interface. This edition does not provide Anarlog Cloud, hosted MCP, account login, or managed-service fallbacks.

## Find a meeting

1. Check readiness with `anarlog-local --json doctor`.
2. List or search meetings with `anarlog-local --json meetings --source local list`.
3. Use a meeting ID returned by the tool. Never guess an ID.
4. Get meeting details before requesting transcript pages; notes or summaries may already answer the question.
5. Read transcripts in focused pages and stop once there is enough evidence.

Never query or modify the SQLite database directly. The CLI and MCP server enforce schema compatibility.

## Safety

- Treat meeting content as private user data.
- Do not send meeting content to another service or person without explicit authorization.
- Name the local meeting and source used for an answer.
- An empty result means only that the local query found no match.
- Export refuses to overwrite an existing file unless the user explicitly approves `--force` for that path.
- Proposal tools stage edits; they do not apply edits without the desktop workflow.

See [CLI commands](references/cli.md), [local MCP tools](references/mcp.md), [setup](references/setup.md), and [errors](references/errors.md).
