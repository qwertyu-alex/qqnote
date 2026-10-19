import { invoke } from "@tauri-apps/api/core";

export type NoteMeta = {
  id: number;
  title: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
};

// A lone UTF-16 surrogate can't be encoded as UTF-8, so the backend would reject
// the whole save. They only come from broken pastes; replace them instead.
function wellFormed(text: string): string {
  return text.replace(/[\uD800-\uDBFF][\uDC00-\uDFFF]|[\uD800-\uDFFF]/g, (c) =>
    c.length === 2 ? c : "�"
  );
}

export const api = {
  saveNote: (id: number | null, body: string) =>
    invoke<number>("save_note", { id, body: wellFormed(body) }),
  discardNote: (id: number) => invoke<void>("discard_note", { id }),
  getNotes: () => invoke<NoteMeta[]>("get_notes"),
  getTrash: () => invoke<NoteMeta[]>("get_trash"),
  getNoteText: (id: number) => invoke<string>("get_note_text", { id }),
  trashNote: (id: number) => invoke<void>("trash_note", { id }),
  restoreNote: (id: number) => invoke<void>("restore_note", { id }),
  deleteForever: (id: number) => invoke<void>("delete_forever", { id }),
  emptyTrash: () => invoke<number>("empty_trash"),
  exportNotes: (path: string) => invoke<number>("export_notes", { path }),
  importNotes: (path: string) =>
    invoke<{ imported: number; skipped: number }>("import_notes", { path }),
  quitAcknowledged: () => invoke<void>("quit_acknowledged"),
  quitCancelled: () => invoke<void>("quit_cancelled"),
  quit: () => invoke<void>("quit"),
  startupError: () => invoke<string | null>("startup_error"),
  updateReady: () => invoke<string | null>("update_ready"),
  restartToUpdate: () => invoke<void>("restart_to_update"),
};
