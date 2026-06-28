# ADR-002: Information Document Model as central contract

**Status:** Accepted  
**Date:** 2026-06-27

## Decision

All adapters produce an `InformationDocument`. The renderer consumes only `InformationDocument`. Raw HTML never reaches the renderer.

## Context

Without a strict boundary, adapters could leak site-specific markup, styles, or logic into the rendering layer. This would gradually recreate the original site interface and undermine the project's core principle.

## Consequences

- The model is versioned (`model_version` field).
- Documents pass validation before rendering.
- Adding a new content source requires only a new adapter — the renderer is unchanged.
- The model can be serialized and reloaded without the original HTML.
- Block types that carry HTML content fail validation and are rejected.
