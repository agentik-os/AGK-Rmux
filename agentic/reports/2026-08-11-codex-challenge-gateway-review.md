# Codex cross-model challenge — omega-gateway (2026-08-11)
Reviewer: OpenAI Codex (gpt-5.6-luna) via direct exec (bwrap read-only sandbox unavailable on this box). Read-only on a disposable clone of main.

Several concrete gaps are emerging despite the route-layer placement being correct: chat accepts an unrestricted filesystem `cwd`, several blocking subprocess wrappers cannot be cancelled on disconnect, and direct-child cleanup does not cover nested provider processes in chat/PDF. I’m validating the exact argv and ownership behavior now, especially session close and account-store races, before ranking them.

## Important findings

### I-1. Chat allows arbitrary filesystem working directories

**Severity:** Important  
**Location:** `src/routes_chat.rs:27-29, 66-85`; `src/chat_driver.rs:46-61`

**Issue:** `POST /v1/chats` accepts `cwd` without validation. Chat turns execute Claude with `current_dir(&meta.cwd)`, allowing an authenticated device to operate in `/`, `/etc`, another project, or another device’s private directory.

**Trigger:**

```http
POST /v1/chats
Authorization: Bearer <valid-token>
Content-Type: application/json

{"agent":"claude","cwd":"/etc"}
```

Then connect to the returned chat WebSocket and submit a message. The Claude process runs with `/etc` as its working directory.

**Suggested fix:** Resolve `cwd` through the same canonical project or home-directory confinement used by `routes_files` and `routes_sessions`. Reject absolute paths outside approved roots, symlink escapes, nonexistent unsafe paths, and NUL bytes.

---

### I-2. Blocking `omega` subprocesses survive client disconnects and have no gateway timeout

**Severity:** Important  
**Locations:** `src/routes_sessions.rs:565-606`; `src/routes_team.rs:157-196`; `src/routes_dispatch.rs:176-210`; `src/routes_oracles.rs:272-312`

**Issue:** These handlers call `spawn_blocking`, then run synchronous `Command::output()` through `omega_cli::run`. Dropping the HTTP handler future does not cancel the blocking task or its child process. None of these calls has a timeout or process-group cleanup.

**Trigger:** Send a request to `/v1/sessions`, `/v1/team`, `/v1/dispatch`, `/v1/oracles/{session}/reap`, or `/resurrect`, then immediately disconnect the client. A hanging or long-running fake/real `omega` process continues running after the request is gone. Repeating this can exhaust subprocesses and blocking-pool capacity.

**Suggested fix:** Use `tokio::process::Command`, assign a dedicated process group, wrap execution in `tokio::time::timeout`, and kill and reap the entire group on cancellation, timeout, and disconnect.

---

### I-3. Chat and PDF cleanup kills only the direct child, not nested provider processes

**Severity:** Important  
**Locations:** `src/chat_driver.rs:204-208, 239-258`; `src/routes_pdf.rs:178-199`

**Issue:** `kill_on_drop(true)` and `child.kill()` only terminate the direct process. Claude may spawn nested processes, and `omega pdf` invokes Node/npm tooling. Neither command is placed in a dedicated process group.

**Trigger:** Configure the executable to spawn a long-lived grandchild, start a chat turn or PDF generation, then disconnect or wait for timeout. The direct child is killed, but the grandchild remains alive.

**Suggested fix:** Set `process_group(0)` or equivalent, capture the PID, and kill the negative process-group ID on timeout and cancellation. Reap the direct child afterward.

---

### I-4. Audit, agent-install, orchestrate, and new-project streams have no outer timeout

**Severity:** Important  
**Locations:** `src/routes_agents.rs:254-358`; `src/routes_audit.rs:243-250`; `src/routes_orchestrate.rs:253-270`; `src/routes_new_project.rs:334-358`

**Issue:** These WebSocket loops detect disconnects and kill their direct process groups, but there is no gateway-level timeout. A child that remains alive but produces no output can hold a connection and permit indefinitely. The new-project code explicitly documents that the bootstrap session continues after the WebSocket closes (`routes_new_project.rs:315-319`).

**Trigger:** Replace or induce a hanging `omega` command, open the stream, and send no further data. The connection and subprocess can remain live indefinitely. For new-project, closing the socket does not stop the created `*-setup` session.

**Suggested fix:** Add explicit bounded execution time. For new-project, define and implement cancellation semantics for the daemon-created setup session, or explicitly expose it as a separate operator-controlled operation.

---

### I-5. Multiple clients can run concurrent turns against the same chat

**Severity:** Important  
**Locations:** `src/routes_chat.rs:183-228`; `src/chat_store.rs:123-148, 145-147`

**Issue:** `chat_permits` is only a global cap. There is no per-chat lock. Two WebSocket connections can read the same provider session ID and concurrently launch turns against it. Transcript writes and metadata updates can race, and the last provider session ID write wins.

**Trigger:** Create one chat, open two authenticated WebSockets for its ID, and submit messages concurrently. Both requests pass the semaphore and start turns against the same persisted chat.

**Suggested fix:** Add a per-chat mutex or an atomic turn state. Serialize provider-session updates and transcript writes per chat, or reject a second active turn with `409`.

---

### I-6. Account registry operations have a lost-update race

**Severity:** Important  
**Locations:** `src/accounts.rs:113-139, 154-165, 168-180`; `src/routes_accounts.rs:64-76`

**Issue:** Account mutations use a read-modify-write sequence with a shared fixed temporary filename, `accounts.json.tmp`, but no process-local or filesystem lock. Concurrent account creation or deletion can overwrite another operation’s registry update while leaving slot directories inconsistent with `accounts.json`.

**Trigger:** Send two concurrent `POST /v1/accounts` requests for different slugs. Both can read the same registry, write different temporary contents, and rename in an order that loses one account.

**Suggested fix:** Serialize registry mutations with a shared mutex and use a unique temporary file plus atomic rename. Reconcile slot-directory creation and registry persistence on failure.

---

### I-7. Session close leaves a user-controlled positional argument unprotected

**Severity:** Important  
**Location:** `src/routes_sessions.rs:226-249`; `src/routes_sessions.rs:94-114`

**Issue:** `valid_session_name` permits names beginning with `-`, but close invokes:

```rust
crate::omega_cli::run(&["kill", &session])
```

There is no `--` separator. A path such as `-x` can be parsed by the CLI as an option rather than a session name.

**Trigger:**

```http
POST /v1/sessions/-x/close
Authorization: Bearer <valid-token>
```

The route validation accepts the name, then passes it directly as the `omega kill` argument.

**Suggested fix:** Use `["kill", "--", session]` if the CLI supports the separator, and reject leading-hyphen names consistently. Apply the same protection to any rmux target argument that can begin with `-`.

## Minor findings

### M-1. Raw subprocess output is returned to clients

**Severity:** Minor  
**Locations:** `src/routes_duo.rs:613-619`; `src/routes_pdf.rs:273-277`; `src/routes_dispatch.rs:214-224`; `src/routes_team.rs:198-206`; `src/routes_sessions.rs:260-271`

**Issue:** Full stdout and stderr are returned in error responses. Provider tools, npm, Node scripts, or future CLI diagnostics can include environment-derived credentials, URLs, file contents, or other secrets.

**Trigger:** Cause an underlying command to emit sensitive data to stderr/stdout and fail. The gateway returns that output in the HTTP response.

**Suggested fix:** Return sanitized user-facing errors. Keep full diagnostics in access-controlled logs with secrets redacted, and never echo provider subprocess output wholesale.

---

### M-2. Account API-key login has no timeout or cancellation path

**Severity:** Minor  
**Location:** `src/routes_accounts.rs:109-142`; `src/account_login.rs:209-231`

**Issue:** The API-key login runs inside `spawn_blocking` and waits indefinitely for `codex`. Client disconnect does not stop the blocking task or child.

**Trigger:** Make the Codex executable block after reading the key, then disconnect the request.

**Suggested fix:** Use an async killable child with a timeout and process-group cleanup. Ensure the API key is cleared from memory where practical.

## Verified defenses

- All protected routes in `src/server.rs:243-361` are registered before `.route_layer(...)` at line 365. Only `/v1/health` and `/v1/pair` are outside authentication.
- WebSocket routes are inside the protected router and therefore pass through the same middleware.
- `/v1/files` and `/v1/files/read` canonicalize both root and target and reject absolute paths, traversal, nonexistent escaped ancestors, and symlink escapes.
- Deposit filenames are sanitized before being joined to disk paths.
- `/v1/pdf/download` reduces input to a basename, restricts the generated filename shape, and applies canonical root confinement.
- Session `dir` validation rejects NUL bytes, parent components, relative paths, and paths outside the real home directory.
- Dispatch, orchestrate, new-project, team, and session creation use `--` for positional arguments.
- Account slugs are constrained to `[a-z0-9-]{1,32}`.
- Config and Telegram status responses redact API keys and bot tokens.
- No Critical auth bypass or direct unauthenticated protected route was found.

## Final assessment

The highest-risk issue is unrestricted chat `cwd`, because a valid device token can cause a provider agent to operate outside the intended project boundary. The next major class is subprocess lifecycle failure: several synchronous endpoints cannot be cancelled, while chat and PDF cleanup does not reach nested children.

Résumé français: aucune route protégée n’a été trouvée sans authentification. Les problèmes importants sont le `cwd` de chat non confiné, les subprocessus synchrones non annulables, les enfants imbriqués non tués, l’absence de timeout sur plusieurs streams, les tours concurrents sur un même chat et la course dans le registre des comptes.
tokens used
160,516
# omega-gateway security review

Scope: `crates/omega-gateway/src/` and `tests/`. Review was read-only. No files were modified.

## Important findings

### I-1. Chat allows arbitrary filesystem working directories

**Severity:** Important  
**Location:** `src/routes_chat.rs:27-29, 66-85`; `src/chat_driver.rs:46-61`

**Issue:** `POST /v1/chats` accepts `cwd` without validation. Chat turns execute Claude with `current_dir(&meta.cwd)`, allowing an authenticated device to operate in `/`, `/etc`, another project, or another device’s private directory.

**Trigger:**

```http
POST /v1/chats
Authorization: Bearer <valid-token>
Content-Type: application/json

{"agent":"claude","cwd":"/etc"}
```

Then connect to the returned chat WebSocket and submit a message. The Claude process runs with `/etc` as its working directory.

**Suggested fix:** Resolve `cwd` through the same canonical project or home-directory confinement used by `routes_files` and `routes_sessions`. Reject absolute paths outside approved roots, symlink escapes, nonexistent unsafe paths, and NUL bytes.

---

### I-2. Blocking `omega` subprocesses survive client disconnects and have no gateway timeout

**Severity:** Important  
**Locations:** `src/routes_sessions.rs:565-606`; `src/routes_team.rs:157-196`; `src/routes_dispatch.rs:176-210`; `src/routes_oracles.rs:272-312`

**Issue:** These handlers call `spawn_blocking`, then run synchronous `Command::output()` through `omega_cli::run`. Dropping the HTTP handler future does not cancel the blocking task or its child process. None of these calls has a timeout or process-group cleanup.

**Trigger:** Send a request to `/v1/sessions`, `/v1/team`, `/v1/dispatch`, `/v1/oracles/{session}/reap`, or `/resurrect`, then immediately disconnect the client. A hanging or long-running fake/real `omega` process continues running after the request is gone. Repeating this can exhaust subprocesses and blocking-pool capacity.

**Suggested fix:** Use `tokio::process::Command`, assign a dedicated process group, wrap execution in `tokio::time::timeout`, and kill and reap the entire group on cancellation, timeout, and disconnect.

---

### I-3. Chat and PDF cleanup kills only the direct child, not nested provider processes

**Severity:** Important  
**Locations:** `src/chat_driver.rs:204-208, 239-258`; `src/routes_pdf.rs:178-199`

**Issue:** `kill_on_drop(true)` and `child.kill()` only terminate the direct process. Claude may spawn nested processes, and `omega pdf` invokes Node/npm tooling. Neither command is placed in a dedicated process group.

**Trigger:** Configure the executable to spawn a long-lived grandchild, start a chat turn or PDF generation, then disconnect or wait for timeout. The direct child is killed, but the grandchild remains alive.

**Suggested fix:** Set `process_group(0)` or equivalent, capture the PID, and kill the negative process-group ID on timeout and cancellation. Reap the direct child afterward.

---

### I-4. Audit, agent-install, orchestrate, and new-project streams have no outer timeout

**Severity:** Important  
**Locations:** `src/routes_agents.rs:254-358`; `src/routes_audit.rs:243-250`; `src/routes_orchestrate.rs:253-270`; `src/routes_new_project.rs:334-358`

**Issue:** These WebSocket loops detect disconnects and kill their direct process groups, but there is no gateway-level timeout. A child that remains alive but produces no output can hold a connection and permit indefinitely. The new-project code explicitly documents that the bootstrap session continues after the WebSocket closes (`routes_new_project.rs:315-319`).

**Trigger:** Replace or induce a hanging `omega` command, open the stream, and send no further data. The connection and subprocess can remain live indefinitely. For new-project, closing the socket does not stop the created `*-setup` session.

**Suggested fix:** Add explicit bounded execution time. For new-project, define and implement cancellation semantics for the daemon-created setup session, or explicitly expose it as a separate operator-controlled operation.

---

### I-5. Multiple clients can run concurrent turns against the same chat

**Severity:** Important  
**Locations:** `src/routes_chat.rs:183-228`; `src/chat_store.rs:123-148, 145-147`

**Issue:** `chat_permits` is only a global cap. There is no per-chat lock. Two WebSocket connections can read the same provider session ID and concurrently launch turns against it. Transcript writes and metadata updates can race, and the last provider session ID write wins.

**Trigger:** Create one chat, open two authenticated WebSockets for its ID, and submit messages concurrently. Both requests pass the semaphore and start turns against the same persisted chat.

**Suggested fix:** Add a per-chat mutex or an atomic turn state. Serialize provider-session updates and transcript writes per chat, or reject a second active turn with `409`.

---

### I-6. Account registry operations have a lost-update race

**Severity:** Important  
**Locations:** `src/accounts.rs:113-139, 154-165, 168-180`; `src/routes_accounts.rs:64-76`

**Issue:** Account mutations use a read-modify-write sequence with a shared fixed temporary filename, `accounts.json.tmp`, but no process-local or filesystem lock. Concurrent account creation or deletion can overwrite another operation’s registry update while leaving slot directories inconsistent with `accounts.json`.

**Trigger:** Send two concurrent `POST /v1/accounts` requests for different slugs. Both can read the same registry, write different temporary contents, and rename in an order that loses one account.

**Suggested fix:** Serialize registry mutations with a shared mutex and use a unique temporary file plus atomic rename. Reconcile slot-directory creation and registry persistence on failure.

---

### I-7. Session close leaves a user-controlled positional argument unprotected

**Severity:** Important  
**Location:** `src/routes_sessions.rs:226-249`; `src/routes_sessions.rs:94-114`

**Issue:** `valid_session_name` permits names beginning with `-`, but close invokes:

```rust
crate::omega_cli::run(&["kill", &session])
```

There is no `--` separator. A path such as `-x` can be parsed by the CLI as an option rather than a session name.

**Trigger:**

```http
POST /v1/sessions/-x/close
Authorization: Bearer <valid-token>
```

The route validation accepts the name, then passes it directly as the `omega kill` argument.

**Suggested fix:** Use `["kill", "--", session]` if the CLI supports the separator, and reject leading-hyphen names consistently. Apply the same protection to any rmux target argument that can begin with `-`.

## Minor findings

### M-1. Raw subprocess output is returned to clients

**Severity:** Minor  
**Locations:** `src/routes_duo.rs:613-619`; `src/routes_pdf.rs:273-277`; `src/routes_dispatch.rs:214-224`; `src/routes_team.rs:198-206`; `src/routes_sessions.rs:260-271`

**Issue:** Full stdout and stderr are returned in error responses. Provider tools, npm, Node scripts, or future CLI diagnostics can include environment-derived credentials, URLs, file contents, or other secrets.

**Trigger:** Cause an underlying command to emit sensitive data to stderr/stdout and fail. The gateway returns that output in the HTTP response.

**Suggested fix:** Return sanitized user-facing errors. Keep full diagnostics in access-controlled logs with secrets redacted, and never echo provider subprocess output wholesale.

---

### M-2. Account API-key login has no timeout or cancellation path

**Severity:** Minor  
**Location:** `src/routes_accounts.rs:109-142`; `src/account_login.rs:209-231`

**Issue:** The API-key login runs inside `spawn_blocking` and waits indefinitely for `codex`. Client disconnect does not stop the blocking task or child.

**Trigger:** Make the Codex executable block after reading the key, then disconnect the request.

**Suggested fix:** Use an async killable child with a timeout and process-group cleanup. Ensure the API key is cleared from memory where practical.

## Verified defenses

- All protected routes in `src/server.rs:243-361` are registered before `.route_layer(...)` at line 365. Only `/v1/health` and `/v1/pair` are outside authentication.
- WebSocket routes are inside the protected router and therefore pass through the same middleware.
- `/v1/files` and `/v1/files/read` canonicalize both root and target and reject absolute paths, traversal, nonexistent escaped ancestors, and symlink escapes.
- Deposit filenames are sanitized before being joined to disk paths.
- `/v1/pdf/download` reduces input to a basename, restricts the generated filename shape, and applies canonical root confinement.
- Session `dir` validation rejects NUL bytes, parent components, relative paths, and paths outside the real home directory.
- Dispatch, orchestrate, new-project, team, and session creation use `--` for positional arguments.
- Account slugs are constrained to `[a-z0-9-]{1,32}`.
- Config and Telegram status responses redact API keys and bot tokens.
- No Critical auth bypass or direct unauthenticated protected route was found.

## Final assessment

The highest-risk issue is unrestricted chat `cwd`, because a valid device token can cause a provider agent to operate outside the intended project boundary. The next major class is subprocess lifecycle failure: several synchronous endpoints cannot be cancelled, while chat and PDF cleanup does not reach nested children.

Résumé français: aucune route protégée n’a été trouvée sans authentification. Les problèmes importants sont le `cwd` de chat non confiné, les subprocessus synchrones non annulables, les enfants imbriqués non tués, l’absence de timeout sur plusieurs streams, les tours concurrents sur un même chat et la course dans le registre des comptes.
