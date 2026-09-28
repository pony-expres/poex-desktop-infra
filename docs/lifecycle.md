# Local lifecycle contract

Pony Expres desktop mirrors the production lifecycle while keeping its runtime implementation independent.

States:

- `active`: a fresh Pony invocation process is executing.
- `idle`: daemon/runtime host is resident, but no tenant actor is retained for reuse.
- `frozen`: an admitted OS process may be suspended for pressure management; no new work enters it.
- `reclaiming`: the host may reclaim/swap pages separately from suspension.
- `draining`: the generation accepts no new work while admitted invocations finish.
- `stopped`: tenant actor/process memory is gone; immutable deployment artifacts remain.

Rules:

1. Tenant actor reuse across invocations is forbidden.
2. Every invocation receives a fresh Pony actor/process security cell.
3. Freeze and memory reclaim are separate operations and must be reported separately.
4. Generation activation is prepare -> validate -> stage -> health-check -> activate -> drain old generation -> retire/rollback.
5. The Rust desktop daemon is the sole machine-lifecycle writer; CLI and desktop app are clients only.
6. Erlang may remain resident as the granddaddy supervisor even though Pony tenant actors are disposable.
7. Invocation payloads travel over stdin or authenticated loopback IPC, never argv.
