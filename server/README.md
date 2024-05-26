# ☄️ bundlrs

*Bundlrs* is a rewrite of [Bundles](https://codeberg.org/SentryTwo/bundles) in Rust without some of the extra features.

Bundlrs is a *super* lightweight and [anonymous](#user-accounts) social markdown platform featuring pastes with custom URLs, quick and full deletion, easy editing, live preview, advanced styling, and [much more](#features)!

For migration from Bundles, please see [#3](https://code.stellular.org/stellular/bundlrs/issues/3).

> Also see [Puffer](https://code.stellular.org/stellular/puffer) and [Guppy](https://code.stellular.org/stellular/guppy)! (required)

## Install

Bundlrs provides build scripts using [just](https://github.com/casey/just). It is required that `bun`, `just`, `redis`, and (obviously) Rust are installed before running.

Build:

```bash
# release (sqlite)
just
# release (mysql)
just build mysql
# release (postgres)
just build postgres
# documentation
just docs
```

Documentation is automatically built when building for release.

Run:

```bash
# test
just test
# release
just run
```

Bundlrs supports the features `sqlite`, `postgres`, and `mysql`. These features dictate which database types will be used.

## Configuration

Bundlrs is configured through flags given when running the server. The following flags are available:

- `--port 0000` optional (defaults to `8080`)
- `--static-dir "/path/to/dir` optional (defaults to `./static`)
- `--db-type "type"` optional (defaults to `sqlite`)

Environment variables:

- `INFO` optional (defaults to `/pub/info`)
- `BODY_EMBED` optional (defaults to nothing)
- `DB_HOST "host"` optional (defaults to `localhost`) (only if `--db-type` is `postgres` or `mysql`)
- `DB_USER "user"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `DB_PASS "pass"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `DB_NAME "name"` **required** (only if `--db-type` is `postgres` or `mysql`)
- `SITE_NAME "name"` optional (defaults to `Bundlrs`)
- `AUTH_REQUIRED` optional (defaults to `(None)`), disables creating pastes without an account
- `REGISTRATION_DISABLED` optional (defaults to `(None)`)

## Features

- [Bundlrs Info Page](https://stellular.net/pub/info)
- [Markdown Info Page](https://stellular.net/pub/markdown)
- [Secondary Formatting Examples](https://stellular.net/37dbdb2096)
- [Templates Info Page](https://stellular.net/pub/templates)
- [SSM Info Page](https://stellular.net/pub/ssm)
- [API Docs](https://stellular.net/api/docs/bundlrs/index.html)

## Boards

Boards are simple forum-like rooms that can be created by users. Boards can be made private by changing `is_private` to `yes` in their configuration page; however this will not stop [users](#user-accounts) with the `Staff` role from viewing and posting on the board.

Boards can be given tags in their configuration. All tags must start with `+` and must be space separated.

## User Accounts

Users can register for an account with just a username. They are given a unique ID which will be used as their password. This ID is stored hashed on the server and cannot be reset.

### User Permissions

- `ManagePastes` - Ability to manage (edit metadata, delete) any paste
- `ManageBoards` - Ability to manage (edit metadata, delete, view even with `is_private`) any board
- `ManageBoardPosts` - Ability to manage (edit tags, delete) any board post
- `EditBoardPosts` - Ability to manage (edit) any board post
- `ManageUsers` - Ability to manage (edit metadata, delete) any user
- `StaffDashboard` - Ability to view the staff dashboard (`/d/staff`), as well as be unable to be viewed in the user manager

Levels should be directly managed by managing entries in the `Logs` table. Levels **must** have a `logtype` value of `level`. Their `content` should be in a JSON-serialized format following the structure defined [here](https://stellular.net/api/docs/bundlrs/db/bundlesdb/struct.RoleLevel.html). Their `elevation` should be an int between `-999` and `1000`. An elevation of `-1000` is used for anonymous users.

Here's an example `content` value for a basic staff role with all permissions:

```json
{
    "elevation":5, "name":"staff", "permissions":[
        "ManagePastes",
        "ManageBoards",
        "ManageBoardPosts",
        "EditBoardPosts",
        "ManageUsers",
        "StaffDashboard"
    ]
}
```
