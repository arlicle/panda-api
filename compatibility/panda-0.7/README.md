# Panda API 0.7 compatibility archive

This directory preserves the Panda API 0.7.4 implementation from code commit
`7deff42` as migration evidence for Panda API 2.0.

The package is deliberately excluded from the Panda API 2.0 Cargo workspace.
It is known not to compile with the recorded modern toolchain because its
unlocked legacy dependency graph resolves an incompatible `actix-http`/time
combination. The archived implementation also contains known path traversal,
upload, authentication, secret-handling, locking, and routing risks.

Do not repair or deploy this package as part of Panda API 2.0. Consult the
[legacy documentation](../../docs/legacy/0.7/README.md), use the sources as
migration fixtures, and use the existing `v0.74` tag when the original repository
layout is required.

`SHA256SUMS` covers the frozen payload inherited from the old repository. This
README and the checksum manifest are archive metadata and are not part of that
payload.
