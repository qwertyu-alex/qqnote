import { api } from "./notes";

// Save this long after typing stops...
const SAVE_DELAY_MS = 300;
// ...but at least this often while typing without pauses.
const MAX_SAVE_DELAY_MS = 3000;
const RETRY_DELAY_MS = 2000;

type Session = {
  // null until the note is first saved
  id: number | null;
  // Created here (⌘T or app start) rather than opened from history.
  isNew: boolean;
  // Text as last saved (or loaded).
  saved: string;
  // Text most recently handed to the save queue; null forces the next flush to save.
  queued: string | null;
};

function newSession(): Session {
  return { id: null, isNew: true, saved: "", queued: "" };
}

export type EditorEvent =
  // The first edit since the last save.
  | { type: "edited" }
  | { type: "saved"; id: number | null }
  | { type: "save-failed"; error: string }
  | { type: "switched"; id: number | null };

/**
 * Owns saving for the editor textarea. The textarea is the source of truth: its value is
 * only ever replaced when a different note is opened, so the browser's undo history stays
 * intact while you type.
 */
export class NoteEditor {
  private session = newSession();
  private timer: number | undefined;
  private firstUnsavedEdit: number | undefined;
  private saves: Promise<void> = Promise.resolve();
  private lastSaveFailed = false;
  private busy: Promise<unknown> = Promise.resolve();

  constructor(
    private readonly textarea: () => HTMLTextAreaElement | null,
    private readonly onEvent: (event: EditorEvent) => void
  ) {}

  get currentId(): number | null {
    return this.session.id;
  }

  /** True while there is text that hasn't been saved yet. */
  get hasUnsavedChanges(): boolean {
    const textarea = this.textarea();
    return this.timer !== undefined || (textarea !== null && textarea.value !== this.session.saved);
  }

  /** True for a note created here that has no text, i.e. there is nothing to save. */
  get isEmptyNewNote(): boolean {
    return this.session.isNew && this.textarea()?.value === "";
  }

  /** Call after every edit. */
  edited(): void {
    const now = Date.now();
    if (this.firstUnsavedEdit === undefined) {
      this.firstUnsavedEdit = now;
      this.onEvent({ type: "edited" });
    }
    window.clearTimeout(this.timer);
    if (now - this.firstUnsavedEdit >= MAX_SAVE_DELAY_MS) {
      void this.flush();
      return;
    }
    this.timer = window.setTimeout(() => void this.flush(), SAVE_DELAY_MS);
  }

  /**
   * Saves the open note now if it changed, and waits for every pending save.
   * Resolves to false if the latest save failed (the text is still in the editor).
   */
  flush(): Promise<boolean> {
    window.clearTimeout(this.timer);
    this.timer = undefined;
    this.firstUnsavedEdit = undefined;

    const textarea = this.textarea();
    const session = this.session;
    if (textarea && textarea.value !== session.queued) {
      const body = textarea.value;
      session.queued = body;
      this.saves = this.saves.then(() => this.persist(session, body));
    }
    return this.saves.then(() => !this.lastSaveFailed);
  }

  private async persist(session: Session, body: string): Promise<void> {
    try {
      if (body === "" && session.isNew) {
        // Emptying a note you just created removes it instead of keeping an empty note.
        if (session.id !== null) {
          await api.discardNote(session.id);
        }
        session.id = null;
      } else {
        session.id = await api.saveNote(session.id, body);
      }
      session.saved = body;
      this.lastSaveFailed = false;
      this.onEvent({ type: "saved", id: session.id });
    } catch (error) {
      console.error("Saving the note failed", error);
      session.queued = null;
      this.lastSaveFailed = true;
      this.onEvent({ type: "save-failed", error: String(error) });
      window.clearTimeout(this.timer);
      this.timer = window.setTimeout(() => void this.flush(), RETRY_DELAY_MS);
    }
  }

  /** Opens a note from history. Resolves to false if the open note couldn't be saved first. */
  open(id: number): Promise<boolean> {
    return this.exclusive(async (textarea) => {
      if (!(await this.flush())) {
        return false;
      }
      const body = await api.getNoteText(id);
      this.replace(textarea, { id, isNew: false, saved: body, queued: body }, body);
      return true;
    });
  }

  /** Starts a new, empty note. Resolves to false if the open note couldn't be saved first. */
  newNote(): Promise<boolean> {
    return this.exclusive(async (textarea) => {
      if (!(await this.flush())) {
        return false;
      }
      this.replace(textarea, newSession(), "");
      return true;
    });
  }

  /**
   * Moves a note to the trash. Deleting the open note saves its latest text first and then
   * starts a new note; deleting any other note leaves the editor alone.
   */
  trash(id: number): Promise<boolean> {
    return this.exclusive(async (textarea) => {
      const isOpen = this.session.id === id;
      if (isOpen && !(await this.flush())) {
        return false;
      }
      await api.trashNote(id);
      if (isOpen) {
        this.replace(textarea, newSession(), "");
      }
      return true;
    });
  }

  private replace(textarea: HTMLTextAreaElement, session: Session, text: string): void {
    this.session = session;
    textarea.value = text;
    textarea.setSelectionRange(0, 0);
    textarea.scrollTop = 0;
    this.onEvent({ type: "switched", id: session.id });
  }

  /** Runs note switches one at a time, with typing paused so no keystroke lands in the wrong note. */
  private exclusive<T>(fn: (textarea: HTMLTextAreaElement) => Promise<T>): Promise<T> {
    const run = async () => {
      const textarea = this.textarea();
      if (!textarea) {
        throw new Error("The editor isn't ready yet");
      }
      textarea.readOnly = true;
      try {
        return await fn(textarea);
      } finally {
        textarea.readOnly = false;
        textarea.focus();
      }
    };
    const result = this.busy.then(run, run);
    this.busy = result.catch(() => undefined);
    return result;
  }
}
