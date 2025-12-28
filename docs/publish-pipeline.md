# Publish Pipeline Spec

This document defines the lifecycle of content from draft to published record.

## Lifecycle States

1. **Draft** (Local Only)
    - Stored in local SQL/captured in UI state.
    - Not visible to PDS or other users.
    - Mutable without restriction.
    - IDs are local UUIDs or temporary placeholders.

2. **Published** (Public / Unlisted / Shared)
    - Signed and committed to the AT Protocol repository.
    - Assigned a permanent `at://` URI.
    - stored in Lexicon-compliant format.
    - **Edits**: Append a new commit replacing the record. History is technically preserved in the repo log but UI typically shows latest.

3. **Deprecated / Tombstoned**
    - User "deletes" the content.
    - **Action**: We replace the record with a minimal "tombstone" or actually delete the record from the repo (RepoOp `delete`).
    - *Note*: Aggregators may still have cached copies.

## Protocol Flow

1. **Auth**: User logs in via OAuth (or app app-password initially).
2. **Format**: App converts internal `Draft` model -> `Lexicon` JSON.
3. **Sign & Commit**:
    - App constructs a repository operation (create/update).
    - Sends to PDS (Personal Data Server).
4. **Confirm**: PDS confirms commit CID.
5. **Index**: App updates local distinct "published" view to match confirmed state.

## Versioning Content

- No git-like branching for content *history* in the MVP.
- "Edit" = "Overwrite".
- Collaborative editing (forking) = "Copy & Publish New".
