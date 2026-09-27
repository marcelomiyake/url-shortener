# Short Form — Frontend Design System

This document records the project-owned handoff from the OpenDesign Neutral Modern prototype. The prototype is a design reference; the Vue application is the maintained implementation. OpenDesign is not a runtime dependency.

> Project documentation index: [Documentation index](../docs/README.md)

## Contents

- [Product feel](#product-feel)
- [Tokens](#tokens)
- [Components and states](#components-and-states)
- [Responsive and accessibility rules](#responsive-and-accessibility-rules)
- [Handoff and review](#handoff-and-review)

## Product feel

Make one task obvious: paste a destination, create a short link, and copy it. Use a quiet, neutral canvas and a single blue action. Avoid dashboards, decorative illustrations, or secondary actions that distract from link creation.

## Tokens

| Purpose | Token |
| --- | --- |
| Page background | `#fafafa` |
| Surface | `#ffffff` |
| Primary text | `#111111` |
| Secondary text | `#6b6b6b` |
| Border | `#e5e5e5` |
| Primary action | `#2f6feb` with white text |
| Success / warning / error | `#17a34a` / `#eab308` / `#dc2626` |
| Typeface | Inter when installed; system sans-serif fallback |
| Code/link display | system monospace |
| Spacing | 4 px base scale: 4, 8, 12, 16, 20, 24, 32, 48, 80 |
| Corners | 8 px controls, 12 px cards |
| Content width | 1200 px maximum; 24 px desktop, 16 px tablet, 12 px phone gutters |

## Components and states

- **Page:** centered content with a concise heading and a single white form card.
- **Destination field:** visible label, 56 px minimum height, clear HTTP(S) example, helper text describing the 2 KiB limit, visible keyboard focus, and an error associated with the field.
- **Create button:** full-width blue primary action with at least 48 px height. Show an accessible busy state and prevent duplicate clicks while a request is in flight.
- **Result:** show the relative short path as selectable monospace text, a same-origin link, and a secondary copy button. Announce success and copy feedback through a polite live region.
- **Errors:** display concise actionable text in an alert region. Do not echo raw server internals or turn a submitted Destination URL into active HTML.
- **Motion:** subtle 150–200 ms transitions; respect `prefers-reduced-motion`.

## Responsive and accessibility rules

- Keep the form one column at every viewport. At narrow widths, stack result text and the copy button.
- Use native labels, button controls, appropriate input type, semantic headings, and visible `:focus-visible` treatment.
- Meet WCAG AA contrast for text and controls. Do not rely on color alone to communicate success or failure.
- Keep hit targets at least 44 px, support keyboard-only use, and preserve zoom/reflow without horizontal scrolling.
- Set page language, descriptive title, and viewport metadata. Keep status messages available to screen readers.

## Handoff and review

The OpenDesign prototype was generated on 2026-09-23 using upstream commit `f166c02636e60bc3e3a474fe79cf269aa7f053ea`. Its design uses the Neutral Modern light palette and a single create/copy panel. The source and rendered layout were reviewed; its prototype behavior was illustrative and is not an API or functional test. This Vue implementation follows the visual hierarchy and tokens while using the approved API contract in `docs/design/url-shortener/openapi.yaml`.
