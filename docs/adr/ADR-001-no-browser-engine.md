# ADR-001: No browser engine

**Status:** Accepted  
**Date:** 2026-06-27

## Decision

Unframe does not use Chromium, Gecko, WebKit, or any WebView as its rendering layer.

## Context

A browser engine provides full compatibility with the modern web but it also executes JavaScript, loads third-party resources, applies site CSS, and runs background timers — all without the user's explicit knowledge.

The project hypothesis is that the majority of informational web content (articles, documentation, blogs, news) can be consumed without a full browser engine.

## Consequences

- We control every pixel of the rendered document.
- Sites that require JavaScript to produce content will return an `Unsupported` result.
- The application has no Chromium process, no V8, no blink rendering.
- Memory and CPU usage is a fraction of a full browser.
- Third-party scripts, ads, and analytics are structurally impossible to execute.
