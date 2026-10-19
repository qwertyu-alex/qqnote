import { memo } from "preact/compat";
import { IconRestore } from "../icons/IconRestore";
import { IconTrash } from "../icons/IconTrash";
import { IconXMark } from "../icons/IconXMark";
import type { NoteMeta } from "../notes";
import { deletedDate, parseTimestamp, shortDate } from "../utils";

/** Rows and date headings have fixed heights (see App.css); the list measures them anyway. */
export const ROW_HEIGHT = 50;
export const HEADING_HEIGHT = 31;

/**
 * The list only gets a note's first 70 characters. The first line is the title and the
 * other lines are the preview, joined with dots and without list bullets.
 */
function splitTitle(text: string): { title: string; preview: string } {
  const [title = "", ...rest] = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");
  // The no-break space keeps each dot on the line before it.
  const preview = rest.map((line) => line.replace(/^[-*•]\s+/, "")).join(" · ");
  return { title, preview };
}

function Title(props: { title: string }) {
  return props.title === "" ? (
    <p class="note-title empty">Empty note</p>
  ) : (
    <p class="note-title">{props.title}</p>
  );
}

export function DateHeading(props: { label: string }) {
  return <p class="date-heading">{props.label}</p>;
}

export const NoteRow = memo(function NoteRow(props: {
  note: NoteMeta;
  selected: boolean;
  onOpen: (id: number) => void;
  onTrash: (id: number) => void;
}) {
  const { note } = props;
  const { title, preview } = splitTitle(note.title);

  return (
    <div class={`note-row${props.selected ? " selected" : ""}`} onClick={() => props.onOpen(note.id)}>
      <Title title={title} />
      <p class="note-meta">
        <span class="note-time">{shortDate(parseTimestamp(note.updated_at))}</span>
        {preview && <span class="note-preview">{preview}</span>}
      </p>
      <div class="row-actions">
        <button
          type="button"
          class="icon-button"
          title="Move to Trash"
          onClick={(e) => {
            // Only trash it; don't open it too.
            e.stopPropagation();
            props.onTrash(note.id);
          }}
        >
          <IconTrash />
        </button>
      </div>
    </div>
  );
});

export function TrashRow(props: {
  note: NoteMeta;
  onRestore: (id: number) => void;
  onDelete: (id: number) => void;
}) {
  const { note } = props;
  const { title } = splitTitle(note.title);
  const deleted = note.deleted_at === null ? "" : deletedDate(parseTimestamp(note.deleted_at));

  return (
    <div class="note-row trashed">
      <Title title={title} />
      <p class="note-meta">
        <span class="note-preview">Deleted {deleted}</span>
      </p>
      <div class="row-actions">
        <button
          type="button"
          class="icon-button"
          title="Restore"
          onClick={() => props.onRestore(note.id)}
        >
          <IconRestore />
        </button>
        <button
          type="button"
          class="icon-button danger"
          title="Delete Forever"
          onClick={() => props.onDelete(note.id)}
        >
          <IconXMark />
        </button>
      </div>
    </div>
  );
}
