# 0009 — Artifact checksums in the lockfile

**Status:** Accepted (1.7.3; the maintainer decided where marketplace hashes
come from)

## Context

A 1.7.2 lock ([ADR 0008](0008-lockfile.md)) pins names, versions, sources and
edges. It does not pin bytes, so the same version can come back as a
different artifact:

- an apt repository or mirror can serve another `.deb` under the same
  version string;
- a zugot recipe's source archive can be re-tagged or edited without a version
  bump.

ADR 0008 left room for a checksum by ignoring unknown keys. The open question
was where each source's hash comes from, because nous never downloads or
hashes an artifact.

- **apt** publishes a SHA-256 for every `.deb`. It is the `SHA256` field of a
  Packages index stanza, which `apt-cache show` prints (Debian repository
  format, "Packages Indices"). apt verifies that hash itself when it
  downloads.
- **A zugot recipe** has `[source] sha256`, the digest of its source archive,
  which takumi checks. 520 of 563 recipes carry one. The other 43 are empty:
  meta-packages with no source, or a hash still marked TODO.
- **Marketplace and Flutter packages** have none that nous can read. nous
  reads `installed/<pkg>/manifest.json`, which has no hash field and which
  nothing in ark or mela writes today. mela keeps a SHA-256 `package_hash`
  for each package, but in its own `<root>/index.json` with a different
  layout (`packages/<name>/`). Whether the native index replaces that path is
  open question 4 of the ark v2 sovereignty path.

## Decision

- **Checksums come from the sources that publish one:**
  - apt: the `SHA256` field of the candidate's stanza;
  - recipes: `sha256`.
- **Marketplace and Flutter packages lock without a checksum until 1.8.x**
  (the maintainer's choice). Reading mela's `index.json` would tie nous to
  mela's format before the unification is decided. A manifest key would
  check nothing, because nothing writes it. The 1.8.x native index is the
  place for these hashes.
- **The form is `sha256:` followed by 64 lowercase hex digits.**
  - This is the OCI image spec's digest grammar, `algorithm ":" encoded`;
    for sha256 the encoded part matches `/[a-f0-9]{64}/`.
  - The algorithm is in the value, so another one can come later.
  - A published digest is trimmed and lowercased. Anything that is not 64
    hex digits gives no checksum. That covers an empty recipe `sha256` and a
    stanza that only dpkg's status file knows, which has no `SHA256`.
- **ResolvedPkg carries it**, as `nous_rp_checksum`: a new field at offset
  56, so the struct is 64 bytes.
  - `resolved_pkg_new` keeps its seven arguments and starts the field at 0;
    `nous_rp_set_checksum` sets it.
  - Its JSON gains `"checksum"` (a string, or `null` for none).
- **The lockfile writes a `checksum` line only when there is one**, between
  `source` and `dependencies`, as Cargo.lock does:

      [[package]]
      name = "curl"
      version = "8.19.0"
      source = "community"
      checksum = "sha256:9e2a…"
      dependencies = ["glibc", "openssl"]

  The format version stays 1: a 1.7.2 reader ignores the line.
- **Parsing:** `checksum` is optional. When present it must be in the exact
  form above, or `NOUS_ERR_INVALID_LOCKFILE` names the package. Locking a
  plan refuses a checksum that is not in that form, too. A different
  algorithm would come with a new format version.
- **Drift:** `NOUS_LOCK_CHECKSUM_CHANGED` (7) is reported when the old lock
  has a checksum, the version and source are the same, and the new one
  differs or is gone. Two cases are not drift:
  - **A checksum the old lock lacks.** Nothing about the bytes was pinned,
    so a 1.7.2 lock keeps passing `nous_resolve_locked` on 1.7.3. Cargo
    refuses this case, but here that would break every 1.7.2 lock in a
    minor release. Regenerating the lock adds the checksums.
  - **A changed version or source.** The new version or source is reported
    on its own, without a second entry for the checksum.

## Consequences

- **A committed lock now catches "same version, other bytes"** for apt
  packages and for recipes with a source hash. The error names the package
  and both checksums.
- **The diff is one-sided:** `nous_lock_diff(was, now)` compares checksums
  only where `was` has one. Full equality takes both directions, and the lock
  fuzz harness checks both.
- **Removing a hash is drift; adding one is not.** Stripping a checksum out
  of a committed lock silently unpins that package, so a review should catch
  a checksum line deleted from the lock. nous cannot tell a hand edit from an
  older lock.
- **apt checksums need apt's package lists.** With no lists (`apt-get
  update` never run, or `/var/lib/apt/lists` removed, as container images
  often do), `apt-cache show` prints only dpkg's status stanza, which has no
  `SHA256`. Checked against debian:12 output in 1.7.3. A lock made on a host
  with lists therefore fails locked mode on a host without them
  (`<name>: checksum sha256:… -> none`). Run `apt-get update` first. Treating
  a missing checksum as a match would let a stripped lock or a stripped host
  unpin every apt package without anyone noticing.
- **Marketplace and Flutter nodes pin no bytes** until the 1.8.x native
  index. The roadmap carries this.
- **No new error kind:** a checksum change is another `NOUS_ERR_LOCK_MISMATCH`.
  A consumer that matches `LockDiffKind` exhaustively must handle the new 7.
- **nous still hashes nothing.** Checking downloaded bytes against the hash
  stays with apt, takumi and ark. nous pins what the source published.
