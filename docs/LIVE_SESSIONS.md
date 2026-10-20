# Live document sessions — Windows desktop workspace

Implemented on 2026-10-20 in `src/document_session.rs`.

The workspace now owns **one live editor instance per open CSV or DOCX document** (DOCX only with `embedded-wordcraft`). Switching tabs preserves in-memory editor state, including unsaved CSV cells and WordCraft's temporary document. Reopening an already-open path reactivates its existing session instead of loading another copy. Failed loads preserve the previously active session. Closing a WordCraft session still requires explicit temporary-copy discard confirmation; closing a CSV session uses its existing discard/recovery UI.

DOCX remains an experimental temporary-copy editor: saving the working copy is **not** saving the original. XLSX, PPTX and PDF sessions remain external-editor references only; HWP/HWPX remains unsupported. These are not represented as live in-process editors.

The session registry is a common lifecycle layer, **not** a claim that GridCraft/DeckCraft/rHWP/PdfCraft are integrated. Next: expose an adapter trait for open/render/save/export/close with capability flags, integrate more engines, and validate Windows GUI/IME and real files.

## Invariants

- Opening an invalid/missing file never drops the currently active session.
- Session paths are canonicalized to avoid duplicate tabs.
- Each open CSV/DOCX tab retains its own editor instance.
- Closing the active session removes only that session and selects another.
- Save As updates the CSV session path without creating a second live editor.
- No automatic overwrite of original DOCX; no silent discard on tab switch.
