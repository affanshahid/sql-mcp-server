# SQL MCP Server

An MCP server for connecting to and running queries on SQL databases. **Supports built-in SSH tunneling.**

Supported databases: MySQL, MariaDB, PostgreSQL, SQLite.

## Install

**npm**

```sh
npm install -g @affanshahid/sql-mcp-server
```

**Shell (macOS / Linux)**

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/affanshahid/sql-mcp-server/releases/latest/download/sql-mcp-server-installer.sh | sh
```

**PowerShell (Windows)**

```sh
powershell -c "irm https://github.com/affanshahid/sql-mcp-server/releases/latest/download/sql-mcp-server-installer.ps1 | iex"
```

**Build From Source**

```sh
cargo install sql-mcp-server --locked
```

## Usage

```json
{
  "mcpServers": {
    "sql": {
      "command": "sql-mcp-server",
      "env": {
        "DATABASE_URL": "postgres://user:pass@host:5432/dbname"
      }
    }
  }
}
```

## SSH tunneling

To reach a database that isn't directly accessible, supply SSH options and the server will open a local forward and connect through it:

```json
{
  "mcpServers": {
    "sql": {
      "command": "sql-mcp-server",
      "env": {
        "DATABASE_URL": "postgres://user:pass@db.internal:5432/dbname",
        "SSH_HOST": "bastion.example.com",
        "SSH_USERNAME": "deploy",
        "SSH_PRIVATE_KEY": "/home/you/.ssh/id_ed25519"
      }
    }
  }
}
```

Authentication also accepts a password (`SSH_PASSWORD`) instead.

## Permissions

By default only `SELECT` is allowed. Use `DATABASE_OPERATIONS` to permit more:

```json
{
  "mcpServers": {
    "sql": {
      "command": "sql-mcp-server",
      "env": {
        "DATABASE_URL": "postgres://user:pass@host:5432/dbname",
        "DATABASE_OPERATIONS": "select,insert,update"
      }
    }
  }
}
```

Possible values: `select`, `insert`, `update`, `delete`, `ddl`.

Additional guards (off by default):

- `DENY_LIMITLESS_SELECT` — reject `SELECT` without `LIMIT`.
- `DENY_BOUNDLESS_UPDATE` — reject `UPDATE` without `WHERE`.
- `DENY_BOUNDLESS_DELETE` — reject `DELETE` without `WHERE`.

## Configuration reference

Every option can be set as either a CLI flag or an environment variable.

| Flag                      | Env                     | Default  |
| ------------------------- | ----------------------- | -------- |
| `-d`, `--database-url`    | `DATABASE_URL`          | —        |
| `-o`, `--operations`      | `DATABASE_OPERATIONS`   | `select` |
| `--deny-limitless-select` | `DENY_LIMITLESS_SELECT` | `false`  |
| `--deny-boundless-update` | `DENY_BOUNDLESS_UPDATE` | `false`  |
| `--deny-boundless-delete` | `DENY_BOUNDLESS_DELETE` | `false`  |
| `-H`, `--host`            | `SSH_HOST`              | —        |
| `-P`, `--port`            | `SSH_PORT`              | `22`     |
| `-u`, `--username`        | `SSH_USERNAME`          | —        |
| `-p`, `--password`        | `SSH_PASSWORD`          | —        |
| `-i`, `--private-key`     | `SSH_PRIVATE_KEY`       | —        |
