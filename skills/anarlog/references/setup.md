# Setup

## Install the local CLI

Open Anarlog Local, then open Settings → Developers and install the bundled CLI. The command is `anarlog-local` and uses Anarlog Local's separate application-data directory.

Run the app once so its local database exists, then verify access:

```bash
anarlog-local --json doctor
anarlog-local --json meetings --source local list --limit 20
```

## Optional local MCP

Start the stdio MCP server on the same computer as the database:

```bash
anarlog-local mcp
```

Configure the MCP client to run that command with the single `mcp` argument, then restart the client. The server reads the local database through supported application APIs.

Anarlog Local does not provide account login, Cloud Sync, hosted snapshots, or hosted MCP. Do not configure a vendor endpoint as a fallback.
