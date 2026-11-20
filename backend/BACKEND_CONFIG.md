# Backend configuration

## Soroban RPC connections

The backend creates a single `rpc.Server` instance, shared by the event
monitor and all contract reads, so connections to the RPC endpoint are reused
through Node's keep-alive global HTTP(S) agent.

| Variable | Default | Description |
|---|---|---|
| `RPC_MAX_SOCKETS` | `10` | Maximum concurrent sockets per host on the global HTTP(S) agent. Requests beyond this limit queue until a socket is free, bounding RPC connection count under load. |

## Metrics

`GET /metrics` returns Prometheus text format with `shipments_total`,
`milestones_confirmed_total`, `rpc_errors_total` and the
`request_duration_seconds` histogram.

## Shutdown

On `SIGTERM`/`SIGINT` the server stops accepting connections, waits for
in-flight requests, stops the RPC poll timers and closes the database. If this
takes longer than 10 s the process exits with code 1.
