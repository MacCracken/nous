# 0005 — Dependency cycles allowed among system (apt) packages only

**Status:** Accepted (1.5.3, maintainer decision D3)

## Context

CLAUDE.md stated "cycle detection is mandatory — never allow circular
dependencies to pass resolution", and nous rejected every cycle. Debian allows
them. Policy §7.2 says "If there is a circular dependency … the dependency loop
must be broken at some point … Otherwise the breaking point is arbitrary"
(<https://www.debian.org/doc/debian-policy/ch-relationships.html>). Real ones
exist. On Debian 12, verified in the 1.5.3 audit:

    libc6:     Depends: libgcc-s1
    libgcc-s1: Depends: gcc-12-base (= 12.2.0-14+deb12u1), libc6 (>= 2.35)

So any transitive resolve that reached libc6 through the system leg, which is
nearly any system package, failed with `NOUS_ERR_DEPENDENCY_CYCLE`.

Options considered:

- **Keep rejecting every cycle.** Transitive system resolution stays broken.
- **Delegate the system closure to apt** (`apt-get -s install`). It is the
  most faithful, since apt's own solver handles cycles, alternatives and
  virtual packages. But it is a larger change, and it removes nous's view of
  the system subgraph. Recorded on the roadmap.
- **Allow cycles among system packages only** (chosen).

## Decision

Cycles are found as strongly connected components (an iterative Tarjan pass,
`_nous_tarjan` in `src/graph.cyr`).

- A cyclic component made only of `SOURCE_SYSTEM` packages is **accepted**.
  Its members install in byte order, which is deterministic. Policy leaves the
  break point arbitrary.
- A cyclic component that contains **any** marketplace, community or Flutter
  package is still `Err(NOUS_ERR_DEPENDENCY_CYCLE)`, with a concrete cycle path.

The CLAUDE.md rule now reads: cycle detection is mandatory; a cycle passes
resolution only when every package in it comes from the system (apt), as
Debian allows, and the reason is recorded in this ADR.

## Consequences

- curl's apt closure (32 packages through libc6 ↔ libgcc-s1) resolves.
  Verified on Debian 12.
- Install order comes from Tarjan's emission order, which puts dependencies
  first. It replaces Kahn's order reversed: both are valid topological orders,
  but they can differ.
- The iterative pass removes the stack-depth crash that the recursive DFS hit
  on 50k+ chains. It also fixes the reported path for a self-loop reached
  through another package (`aa -> aa`, not `xx -> aa -> aa`).
- `dep_graph_detect_cycle` (public) uses the same pass and still reports any
  cycle, whatever its sources. The source-aware acceptance lives in
  `finalize_graph`.
- A cycle between a marketplace package and a system package is still
  rejected, so a marketplace manifest cannot exploit the exemption.
