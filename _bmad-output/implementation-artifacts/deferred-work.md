# Deferred Work

## Hostile concurrent project-file mutation

- **Source:** multi-file project review, 2026-08-10.
- **Deferred boundary:** the std-only loader rejects discovered symlinks, enforces canonical-root containment, binds reads to an opened handle, and rechecks file identity; it does not promise an atomic cross-platform `nofollow` open or a snapshot while another process rewrites the same inode.
- **Trigger to revisit:** a real untrusted concurrent-filesystem, watch, incremental-analysis, or editor-service use case.
- **Required before implementation:** define the threat model and portability target, then approve any OS-specific or third-party filesystem primitive.
