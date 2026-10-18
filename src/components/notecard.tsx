import { memo } from "preact/compat";
import { IconCloseCircle } from "../icons/IconCloseCircle";
import type { NoteMeta } from "../notes";
import { getRelativeDate } from "../utils";

function NoteTitle(props: { title: string }) {
  const empty = props.title.trim() === "";
  return (
    <p class={`shh note-title${empty ? " empty" : ""}`}>{empty ? "Empty note" : props.title}</p>
  );
}

export const NoteCard = memo(function NoteCard(props: {
  note: NoteMeta;
  selected: boolean;
  onOpen: (id: number) => void;
  onTrash: (id: number) => void;
}) {
  const { note } = props;

  return (
    <div class={"note-card-container"}>
      <div
        class={`note-card shh${props.selected ? " selected" : ""}`}
        onClick={() => props.onOpen(note.id)}
      >
        <div class="note-text">
          <NoteTitle title={note.title} />
          <p class={"shh note-created-at"}>
            {getRelativeDate(new Date(`${note.updated_at}Z`), 7)}
          </p>
        </div>
      </div>
      <IconCloseCircle
        className={"close-button-icon shh"}
        onClick={() => props.onTrash(note.id)}
      />
    </div>
  );
});

export function TrashCard(props: {
  note: NoteMeta;
  onRestore: (id: number) => void;
  onDelete: (id: number) => void;
}) {
  const { note } = props;

  return (
    <div class="note-card trashed shh">
      <div class="note-text">
        <NoteTitle title={note.title} />
        <p class={"shh note-created-at"}>
          Deleted {getRelativeDate(new Date(`${note.deleted_at}Z`), 7)}
        </p>
      </div>
      <div class="trash-actions">
        <button type="button" class="link-button" onClick={() => props.onRestore(note.id)}>
          Restore
        </button>
        <button type="button" class="link-button danger" onClick={() => props.onDelete(note.id)}>
          Delete forever
        </button>
      </div>
    </div>
  );
}
