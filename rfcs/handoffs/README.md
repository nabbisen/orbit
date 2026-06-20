# Developer Handoffs

This directory holds **developer handoffs**: implementation-ready companions to
the RFCs in `rfcs/proposed/`. The RFC answers *what and why* (requirement /
external design); the handoff answers *how* (internal / program design), so an
implementer can go straight to coding per the project workflow:

> Requirement (RFC) → External Design → **Internal/Program Design (handoff)** →
> Implementation → Testing

## Convention

- One handoff per RFC, named `HANDOFF-0NN-<slug>.md`.
- Each handoff is self-contained: exact crates/files touched, function
  signatures, an ordered task list, the test plan, and a definition-of-done
  checklist.
- Handoffs assume the **release-discipline rule**: all work lands in the current
  release version; no version number is created without explicit instruction.
- Handoffs respect the **boundary rules**: `orbok-ui` does no filesystem or
  database access (RFC-027); platform I/O (OS theme / locale / reduce-motion
  probing, settings persistence) lives in `orbok-app`.
- Every change keeps the build **warning-free** (including `--tests`) and the
  full suite green before the step is considered done.

## The design-system program (RFC-032 → 035)

| RFC | Handoff | Theme |
|-----|---------|-------|
| 032 | HANDOFF-032 | Design token foundation + theming (substrate) |
| 033 | HANDOFF-033 | Component primitive migration (snora as primitive gateway) |
| 034 | HANDOFF-034 | Accessibility conformance (WCAG 2.1 AA) |
| 035 | HANDOFF-035 | Inclusive design (text scale, reduced motion, CVD-safe, i18n formatting) |

**Sequencing is strict:** 032 is a hard prerequisite for 033/034/035 because it
threads tokens everywhere; 033 must precede 034 (accessibility rides on the
accessible primitives); 035 can begin once 032 lands and overlaps 033/034 on the
Settings surface. A natural release boundary is "032+033 land together"
(foundation + components are the visible payoff), then "034+035" as the
accessibility/inclusivity release.
