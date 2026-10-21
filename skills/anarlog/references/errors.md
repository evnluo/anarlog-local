# Errors

## Meeting not found

List or search local meetings again and use a returned ID. Do not retry a guessed ID.

## No meetings returned

An empty list means no local meeting matched the query. It is not evidence about any other installation or service.

## Database not found

Run `anarlog-local --json doctor`. Ask the user to open Anarlog Local once if its independent local database does not exist.

## Database operation failed

Confirm the desktop app and CLI come from compatible Anarlog Local builds. Report the error; do not modify the database with SQL or fall back to an Anarlog hosted service.

## Export output exists

Choose a new path. Use `--force` only when the user explicitly approves replacing that exact file.

## MCP server exits

Run `anarlog-local --json doctor` and a local meeting list to distinguish database readiness from MCP client configuration. Preserve the returned error message and code.
