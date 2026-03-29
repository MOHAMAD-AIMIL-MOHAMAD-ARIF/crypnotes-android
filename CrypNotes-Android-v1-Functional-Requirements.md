# CrypNotes Android v1 — Functional Requirements

|Functional requirement|Description|
|-|-|
|FR-01: App themes|The app must support both light and dark themes.|
|FR-02: Note background colors|Users must be able to set per-note background colors.|
|FR-03: Labels|The app must support labels with a many-to-many relationship between notes and labels (via `Label` and `NoteLabel`).|
|FR-04: Sorting|Notes must be sortable by date created, date modified, and custom/manual order.|
|FR-05: Pinning|Users must be able to pin/unpin notes, and pinned notes must appear at the top.|
|FR-06: Archive|The app must provide an Archive folder for notes.|
|FR-07: Trash retention|The app must provide a Trash folder and permanently delete trashed notes after exactly 30 × 24 hours from the trash timestamp.|
|FR-08: Auto-lock timer|The vault must auto-lock when the app exits/backgrounds, with selectable timer values: Immediate, 30s, 1m, 2m, 5m.|
|FR-09: Zeroization on lock|On vault lock, decrypted in-memory notes/cache must be zeroized.|
|FR-10: Quick unlock option|Users must be able to enable quick unlock using biometric/device credential instead of master password.|
|FR-11: Unlock fallback|If biometric authentication fails, fallback to device credential must occur only when that setting is enabled.|
|FR-12: Invalidated device key recovery|If DK is invalidated, the app must support password-based recovery by deriving KEK (Argon2id), decrypting `vault\_key\_blob` to recover DEK, and re-establishing biometric unlock with a new DK and `local\_wrapped\_dek`.|
|FR-13: Rich text formatting|The editor must support bold, italic, underline, highlight, headings H1/H2/H3, checklist, bulleted list, and numbered list.|
|FR-14: Text/checklist conversion|Users must be able to convert note text to checklist and revert checklist back to text; conversion applies to all text in the note.|
|FR-15: Undo/Redo|The editor must support Undo and Redo during note editing.|
|FR-16: Title behavior|Notes must have an optional explicit `title`; if empty, the display title must be derived from the first paragraph at the top of the note.|
|FR-17: Note size limit|Notes must enforce a maximum size of 20,000 characters.|
|FR-18: Image/photo attachments|Image/photo attachments must enforce a 5 MB hard limit, pre-encryption resize to max dimension 1920 px, JPEG compression around 80–85%, and storage of both encrypted original blob and encrypted thumbnail blob (target 256 px).|
|FR-19: Audio attachments|Audio recordings must enforce a hard maximum duration of 3 minutes at recorder layer.|
|FR-20: Attachment lifecycle in Trash|Attachments linked to trashed notes must remain stored until trash expiry, then be permanently deleted.|
|FR-21: Reminder entity model|Reminders must be first-class objects, not embedded only in note metadata.|
|FR-22: Reminder recurrence|Recurrence options must include None, Daily, Weekly, Monthly, and Yearly.|
|FR-23: Reminder time semantics|Reminder recurrence must be timezone-anchored with DST handling: missing local times shift forward and repeated local times trigger once.|
|FR-24: Reminder suppression by note state|Reminders must be disabled when associated notes are archived or trashed.|



