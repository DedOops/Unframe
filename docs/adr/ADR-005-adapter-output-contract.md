# ADR-005: Adapter output contract

**Status:** Accepted  
**Date:** 2026-06-27

## Decision

Every adapter returns one of four outcomes: `Success`, `PartialSuccess`, `Unsupported`, or `Failed`. The adapter never modifies the UI or issues network requests.

## Context

Adapters need a way to communicate degrees of success without breaking the rendering pipeline. A strict four-state contract keeps the adapter boundary clean and makes failure handling predictable.

## Consequences

- `Success` — content extracted, no warnings.
- `PartialSuccess` — content extracted with warnings (e.g. truncation, charset fallback).
- `Unsupported` — the source requires capabilities the client does not have (e.g. JavaScript-only SPA).
- `Failed` — a technical error occurred (network, parse, internal).
- The application shell handles all four states; the adapter has no UI access.
- New adapters must conform to this contract to be accepted into the pipeline.
