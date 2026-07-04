# MCP Compatibility Matrix

ContextClean exposes a stdio MCP server through:

```bash
ctxclean mcp
```

The server writes JSON-RPC protocol messages to stdout only. Diagnostics and errors are kept on stderr by the CLI wrapper, which keeps it safe for stdio MCP clients.

## Client Matrix

| Client | Status | Config shape | Smoke coverage |
|---|---|---|---|
| Claude Desktop | Compatible | command + args stdio server | `initialize`, `tools/list`, `tools/call` |
| Cursor | Compatible | MCP server command entry | `initialize`, `tools/list`, `contextclean_clean` |
| Codex | Compatible | local stdio MCP command | `initialize`, `tools/list`, `contextclean_report` |
| VS Code MCP clients | Compatible | command + args server object | `initialize`, `tools/list`, `tools/call` |
| Generic stdio MCP clients | Compatible | spawn `ctxclean mcp` | newline-delimited JSON-RPC |

## Tools

- `contextclean_clean`
- `contextclean_report`

Both tools accept exactly one source:

- `text` or `input`
- `path`

Common options:

- `mode`: `light`, `standard`, `aggressive`
- `format`: `text`, `markdown`, `json`
- `maxTokens`
- `fit`: `gpt-4.1`, `claude-sonnet`, `gemini-pro`
- `stripComments`
- `redactSecrets`
- `includeSensitive`

## Client Config Examples

Claude Desktop / Cursor / VS Code style:

```json
{
  "mcpServers": {
    "contextclean": {
      "command": "ctxclean",
      "args": ["mcp"]
    }
  }
}
```

Absolute-path Windows example:

```json
{
  "mcpServers": {
    "contextclean": {
      "command": "C:\\Users\\you\\.cargo\\bin\\ctxclean.exe",
      "args": ["mcp"]
    }
  }
}
```

## Smoke Transcript

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"contextclean_clean","arguments":{"text":"<script>noise()</script><main><h1>Keep MCP</h1></main>","format":"text"}}}
{"jsonrpc":"2.0","id":4,"method":"shutdown","params":{}}
```

Expected output includes:

- `protocolVersion`
- `contextclean_clean`
- `contextclean_report`
- `Keep MCP`
- no `noise()`

The CLI smoke test `mcp_lists_tools_and_cleans_inline_input` exercises this transcript.

## Sensitive Paths And Redaction

MCP `path` input uses the same repo reader as the CLI:

- `.gitignore` is respected
- `.ctxcleanignore` is respected
- generated directories such as `.git`, `node_modules`, `target`, `dist`, `build`, and caches are skipped
- secret-like values are redacted by default
- sensitive paths such as `.env`, private keys, certificates, and credential folders require `includeSensitive: true`

`includeSensitive: true` only allows reading sensitive paths. It does not disable redaction. Set `redactSecrets: false` only when the client deliberately wants raw sensitive output.
