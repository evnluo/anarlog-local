# Local MCP tools

Start the local stdio server with:

```bash
anarlog-local mcp
```

A generic client configuration is:

```json
{
  "mcpServers": {
    "anarlog-local": {
      "command": "anarlog-local",
      "args": ["mcp"]
    }
  }
}
```

Local tools include meeting and folder listing, meeting details, bounded transcript pages, recurring history, export, recording import, and proposal staging. Use IDs returned by list/search tools. Transcript pages default to a bounded response; continue from the returned offset only as needed.

Proposal tools create or decline staged edits; they do not silently apply changes. Recording import is only for an explicit user request and may invoke the local or BYOK providers configured in Anarlog Local.

This edition has no hosted MCP endpoint or Cloud fallback.
