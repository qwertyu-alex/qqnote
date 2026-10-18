import { listen } from "@tauri-apps/api/event";
import { downloadDir, join } from "@tauri-apps/api/path";
import { ask, open, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type { KeyboardEventHandler } from "preact";
import { useCallback, useEffect, useMemo, useRef, useState } from "preact/compat";
import "./App.css";
import { NoteCard, TrashCard } from "./components/notecard";
import { StatusBar } from "./components/status-bar";
import { Toast, type ToastMessage } from "./components/toast";
import { type EditorEvent, NoteEditor } from "./editor";
import { useKeyPress } from "./hooks";
import { api, type NoteMeta } from "./notes";

const NOTICE_MS = 1500;
const TOAST_MS = 6000;
const QUIT_SAVE_TIMEOUT_MS = 5000;
const UNSAVED_MESSAGE = "Couldn't save the open note, so it's still open.";

function countWords(text: string): number {
  const trimmed = text.trim();
  return trimmed === "" ? 0 : trimmed.split(/\s+/).length;
}

function today(): string {
  const d = new Date();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${month}-${day}`;
}

function App() {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [currentId, setCurrentId] = useState<number | null>(null);
  const [notes, setNotes] = useState<NoteMeta[]>([]);
  const [trash, setTrash] = useState<NoteMeta[]>([]);
  const [showHistory, setShowHistory] = useState(false);
  const [showTrash, setShowTrash] = useState(false);
  const [stats, setStats] = useState({ chars: 0, words: 0 });
  const [notice, setNotice] = useState<string | null>(null);
  const [toast, setToast] = useState<ToastMessage | null>(null);
  // Editing stays off until the backend confirms the database opened.
  const [startup, setStartup] = useState<{ ready: boolean; error: string | null }>({
    ready: false,
    error: null,
  });

  const showHistoryRef = useRef(showHistory);
  showHistoryRef.current = showHistory;
  const canEdit = useRef(false);
  canEdit.current = startup.ready && startup.error === null;
  const saveFailing = useRef(false);
  const noticeTimer = useRef<number | undefined>(undefined);
  const toastTimer = useRef<number | undefined>(undefined);
  const statsTimer = useRef<number | undefined>(undefined);

  const onEditorEvent = useRef<(event: EditorEvent) => void>(() => {});
  const editor = useMemo(
    () => new NoteEditor(() => textareaRef.current, (event) => onEditorEvent.current(event)),
    []
  );

  const flashNotice = useCallback((text: string) => {
    window.clearTimeout(noticeTimer.current);
    setNotice(text);
    noticeTimer.current = window.setTimeout(() => setNotice(null), NOTICE_MS);
  }, []);

  const showToast = useCallback((message: ToastMessage) => {
    window.clearTimeout(toastTimer.current);
    setToast(message);
    toastTimer.current = window.setTimeout(() => setToast(null), TOAST_MS);
  }, []);

  const updateStats = useCallback(() => {
    const text = textareaRef.current?.value ?? "";
    setStats({ chars: text.length, words: countWords(text) });
  }, []);

  const refreshNotes = useCallback(async () => {
    try {
      setNotes(await api.getNotes());
    } catch (error) {
      console.error("Loading notes failed", error);
    }
  }, []);

  const refreshTrash = useCallback(async () => {
    try {
      setTrash(await api.getTrash());
    } catch (error) {
      console.error("Loading the trash failed", error);
    }
  }, []);

  onEditorEvent.current = (event) => {
    switch (event.type) {
      case "saved":
        setCurrentId(event.id);
        if (saveFailing.current) {
          saveFailing.current = false;
          setNotice(null);
        }
        if (showHistoryRef.current) {
          void refreshNotes();
        }
        break;
      case "save-failed":
        saveFailing.current = true;
        window.clearTimeout(noticeTimer.current);
        setNotice("Couldn't save, retrying…");
        break;
      case "switched":
        setCurrentId(event.id);
        updateStats();
        void refreshNotes();
        break;
    }
  };

  const openNote = useCallback(async (id: number) => {
    try {
      if (!(await editor.open(id))) {
        showToast({ text: UNSAVED_MESSAGE });
      }
    } catch (error) {
      showToast({ text: `Couldn't open the note: ${error}` });
    }
  }, []);

  const newNote = useCallback(async () => {
    try {
      if (!(await editor.newNote())) {
        showToast({ text: UNSAVED_MESSAGE });
      }
    } catch (error) {
      showToast({ text: `Couldn't start a new note: ${error}` });
    }
  }, []);

  const restoreNote = useCallback(async (id: number, reopen: boolean) => {
    try {
      await api.restoreNote(id);
      if (reopen && !(await editor.open(id))) {
        showToast({ text: UNSAVED_MESSAGE });
      }
    } catch (error) {
      showToast({ text: `Couldn't restore the note: ${error}` });
    }
    void refreshNotes();
    void refreshTrash();
  }, []);

  const trashNote = useCallback(async (id: number) => {
    const wasOpen = editor.currentId === id;
    try {
      if (!(await editor.trash(id))) {
        showToast({ text: "Couldn't save the note, so it wasn't deleted." });
        return;
      }
    } catch (error) {
      showToast({ text: `Couldn't delete the note: ${error}` });
      return;
    }
    void refreshNotes();
    showToast({
      text: "Moved to Trash",
      action: { label: "Undo", run: () => void restoreNote(id, wasOpen) },
    });
  }, []);

  const deleteForever = useCallback(async (id: number) => {
    const confirmed = await ask("Delete this note forever? This can't be undone.", {
      title: "qqnote",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (!confirmed) {
      return;
    }
    try {
      await api.deleteForever(id);
    } catch (error) {
      showToast({ text: `Couldn't delete the note: ${error}` });
    }
    void refreshTrash();
  }, []);

  const emptyTrash = useCallback(async () => {
    const confirmed = await ask(
      `Delete ${trash.length === 1 ? "1 note" : `${trash.length} notes`} forever? This can't be undone.`,
      { title: "qqnote", kind: "warning", okLabel: "Empty Trash", cancelLabel: "Cancel" }
    );
    if (!confirmed) {
      return;
    }
    try {
      await api.emptyTrash();
    } catch (error) {
      showToast({ text: `Couldn't empty the trash: ${error}` });
    }
    void refreshTrash();
  }, [trash.length]);

  const handleExport = useCallback(async () => {
    try {
      const path = await saveDialog({
        defaultPath: await join(await downloadDir(), `qqnote-export-${today()}.json`),
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) {
        return;
      }
      await editor.flush();
      const count = await api.exportNotes(path);
      showToast({ text: `Exported ${count === 1 ? "1 note" : `${count} notes`}` });
    } catch (error) {
      showToast({ text: `Export failed: ${error}` });
    }
  }, []);

  const handleImport = useCallback(async () => {
    try {
      const path = await open({ filters: [{ name: "JSON", extensions: ["json"] }] });
      if (!path) {
        return;
      }
      const { imported, skipped } = await api.importNotes(path);
      const skippedText = skipped ? `, skipped ${skipped} empty or already here` : "";
      showToast({ text: `Imported ${imported === 1 ? "1 note" : `${imported} notes`}${skippedText}` });
      void refreshNotes();
      void refreshTrash();
    } catch (error) {
      showToast({ text: `Import failed: ${error}` });
    }
  }, []);

  // Sent by the backend before quitting or closing the window.
  const flushAndQuit = useCallback(async () => {
    await api.quitAcknowledged();
    const timeout = new Promise<boolean>((resolve) =>
      window.setTimeout(() => resolve(false), QUIT_SAVE_TIMEOUT_MS)
    );
    const saved = await Promise.race([editor.flush(), timeout]);
    if (
      saved ||
      (await ask("qqnote couldn't save the open note. Quit anyway and lose the unsaved changes?", {
        title: "qqnote",
        kind: "warning",
        okLabel: "Quit Anyway",
        cancelLabel: "Cancel",
      }))
    ) {
      await api.quit();
    } else {
      await api.quitCancelled();
    }
  }, []);

  useKeyPress(["b"], () => {
    setShowHistory((prev) => !prev);
  });

  useKeyPress(["s"], async (e) => {
    e.preventDefault();
    if (!canEdit.current) {
      return;
    }
    if (editor.isEmptyNewNote) {
      flashNotice("Nothing to save");
    } else if (await editor.flush()) {
      flashNotice("Saved");
    }
  });

  useKeyPress(["t"], (e) => {
    // File ▸ New Note has the same shortcut; handle the keypress once, here.
    e.preventDefault();
    if (canEdit.current) {
      void newNote();
    }
  });

  useEffect(() => {
    if (showHistory) {
      void refreshNotes();
    } else {
      setShowTrash(false);
    }
  }, [showHistory]);

  useEffect(() => {
    if (showTrash) {
      void refreshTrash();
    }
  }, [showTrash]);

  useEffect(() => {
    api.startupError().then(
      (error) => setStartup({ ready: true, error }),
      (error) => setStartup({ ready: true, error: String(error) })
    );
  }, []);

  // Add menu event listeners
  useEffect(() => {
    const unlisteners = [
      listen("export", () => void handleExport()),
      listen("import", () => void handleImport()),
      listen("new_note", () => {
        if (canEdit.current) {
          void newNote();
        }
      }),
      listen("flush-and-quit", () => void flushAndQuit()),
    ];
    // Leaving the window (another app, the Dock, logging out) saves right away.
    const onBlur = () => void editor.flush();
    window.addEventListener("blur", onBlur);

    return () => {
      for (const unlisten of unlisteners) {
        void unlisten.then((fn) => fn());
      }
      window.removeEventListener("blur", onBlur);
    };
  }, []);

  function handleInput() {
    editor.edited();
    window.clearTimeout(statsTimer.current);
    statsTimer.current = window.setTimeout(updateStats, 150);
  }

  const handleKeyDown: KeyboardEventHandler<HTMLTextAreaElement> = (e) => {
    if (e.key === "Tab" && !e.isComposing) {
      e.preventDefault();
      // Inserting through the browser's editing pipeline keeps it undoable, like typing.
      // Assigning textarea.value directly would wipe the undo history.
      document.execCommand("insertText", false, "\t");
    }
  };

  return (
    <div
      style={{
        width: "100%",
        border: "none",
        outline: "none",
        display: "flex",
        overflow: "hidden",
        flex: 1,
        height: "100%",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: "10px",
          zIndex: 9999,
          borderTopLeftRadius: "var(--border-radius)",
          borderTopRightRadius: "var(--border-radius)",
        }}
      />
      <div
        style={{
          flex: 1,
          minWidth: 0,
          display: "flex",
          flexDirection: "column",
        }}
      >
        {startup.error ? (
          <div class="startup-error">
            <p>qqnote couldn't open its database, so editing is turned off to keep your notes safe.</p>
            <p class="startup-error-detail">{startup.error}</p>
          </div>
        ) : (
          <textarea
            placeholder="⌘ + b to show history"
            onInput={handleInput}
            onKeyDown={handleKeyDown}
            id="note-text-area"
            autoComplete={"off"}
            // Plain text only: no spelling underlines, and macOS doesn't rewrite quotes,
            // dashes or words as you type.
            spellcheck={false}
            autocorrect="off"
            autocapitalize="off"
            // biome-ignore lint/a11y/noAutofocus: <explanation>
            autoFocus={true}
            readOnly={!startup.ready}
            style={{
              fontSize: "1em",
              flex: 1,
              minHeight: 0,
              overscrollBehavior: "contain",
              width: "100%",
              margin: "0px",
              padding: "1em",
              outline: "none",
              border: "none",
              resize: "none",
              boxSizing: "border-box",
              backgroundColor: "transparent",
              borderRadius: "var(--border-radius)",
            }}
            ref={textareaRef}
          />
        )}
        <StatusBar chars={stats.chars} words={stats.words} notice={notice} />
      </div>
      {showHistory && (
        <div id="history-container">
          <div id="history-content-container">
            <div class="history-header">
              {showTrash ? (
                <>
                  <button type="button" class="link-button" onClick={() => setShowTrash(false)}>
                    ← Notes
                  </button>
                  {trash.length > 0 && (
                    <button type="button" class="link-button danger" onClick={emptyTrash}>
                      Empty Trash
                    </button>
                  )}
                </>
              ) : (
                <button type="button" class="link-button" onClick={() => setShowTrash(true)}>
                  Trash
                </button>
              )}
            </div>
            {showTrash ? (
              trash.length === 0 ? (
                <p class="history-empty">Trash is empty</p>
              ) : (
                trash.map((n) => (
                  <TrashCard
                    key={n.id}
                    note={n}
                    onRestore={(id) => void restoreNote(id, false)}
                    onDelete={deleteForever}
                  />
                ))
              )
            ) : (
              notes.map((n) => (
                <NoteCard
                  key={n.id}
                  note={n}
                  selected={currentId === n.id}
                  onOpen={openNote}
                  onTrash={trashNote}
                />
              ))
            )}
          </div>
        </div>
      )}
      {toast && <Toast message={toast} onDismiss={() => setToast(null)} />}
    </div>
  );
}

export default App;
