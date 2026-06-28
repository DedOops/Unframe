# ADR-003: Single Network Gateway

**Status:** Accepted  
**Date:** 2026-06-27

## Decision

All network requests go through `unframe-network::NetworkGateway`. Adapters and the renderer cannot make direct network calls.

## Context

Without a single entry point for networking, any component could silently issue requests — loading tracking pixels, analytics endpoints, or third-party resources. Auditing and controlling network activity would become impossible.

## Consequences

- Every request is logged in `NetworkAudit`.
- Limits are applied uniformly: 5 redirects max, 10 MB response cap, 30 s timeout.
- Cookies are disabled.
- Background requests are structurally prevented.
- The Document Passport can show an accurate and complete network summary.
