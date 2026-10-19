import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import { IconChevronLeft } from "../icons/IconChevronLeft";
import { IconCompose } from "../icons/IconCompose";
import { IconTrash } from "../icons/IconTrash";
import type { NoteMeta } from "../notes";
import { dateGroup, parseTimestamp } from "../utils";
import { DateHeading, HEADING_HEIGHT, NoteRow, ROW_HEIGHT, TrashRow } from "./note-row";
import { VirtualList } from "./virtual-list";

type Item = { heading: string } | { note: NoteMeta };

const numberFormat = new Intl.NumberFormat();

/** The notes, newest first, with a heading wherever the date group changes. */
function withHeadings(notes: NoteMeta[]): Item[] {
  const now = new Date();
  const items: Item[] = [];
  let group: string | null = null;
  for (const note of notes) {
    const next = dateGroup(parseTimestamp(note.updated_at), now);
    if (next !== group) {
      items.push({ heading: next });
      group = next;
    }
    items.push({ note });
  }
  return items;
}

/** The sidebar: every note under date headings, or the trash. */
export function History(props: {
  notes: NoteMeta[];
  trash: NoteMeta[];
  showTrash: boolean;
  currentId: number | null;
  onShowTrash: (show: boolean) => void;
  onNewNote: () => void;
  onOpen: (id: number) => void;
  onTrash: (id: number) => void;
  onRestore: (id: number) => void;
  onDeleteForever: (id: number) => void;
  onEmptyTrash: () => void;
}) {
  const listRef = useRef<HTMLDivElement>(null);
  const items = useMemo(() => withHeadings(props.notes), [props.notes]);
  // A hairline under the header while notes are scrolled beneath it.
  const [scrolled, setScrolled] = useState(false);
  useEffect(() => setScrolled(false), [props.showTrash]);
  const header = `history-header${scrolled ? " scrolled" : ""}`;

  return (
    <div id="history-container">
      {props.showTrash ? (
        <div class={header}>
          <div class="history-toolbar">
            <button type="button" class="back-button" onClick={() => props.onShowTrash(false)}>
              <IconChevronLeft />
              Notes
            </button>
            {props.trash.length > 0 && (
              <button type="button" class="text-button danger" onClick={props.onEmptyTrash}>
                Empty Trash
              </button>
            )}
          </div>
          <div class="history-toolbar">
            <h2>Trash</h2>
            <span class="history-count">{numberFormat.format(props.trash.length)}</span>
          </div>
          <p class="history-hint">
            Restore a note to put it back. Emptying the trash deletes these forever.
          </p>
        </div>
      ) : (
        <div class={header}>
          <div class="history-toolbar">
            <h2>Notes</h2>
            <span class="history-count">{numberFormat.format(props.notes.length)}</span>
            <div class="history-tools">
              <button type="button" class="icon-button" title="New Note (⌘T)" onClick={props.onNewNote}>
                <IconCompose />
              </button>
              <button
                type="button"
                class="icon-button"
                title="Trash"
                onClick={() => props.onShowTrash(true)}
              >
                <IconTrash />
              </button>
            </div>
          </div>
        </div>
      )}
      {/* Keyed so switching between the notes and the trash starts at the top, with fresh heights. */}
      <div
        class="history-list"
        key={props.showTrash ? "trash" : "notes"}
        ref={listRef}
        onScroll={(e) => setScrolled(e.currentTarget.scrollTop > 0)}
      >
        {props.showTrash ? (
          props.trash.length === 0 ? (
            <p class="history-empty">Trash is empty</p>
          ) : (
            <VirtualList
              scrollRef={listRef}
              items={props.trash}
              getKey={(note) => note.id}
              estimateHeight={() => ROW_HEIGHT}
            >
              {(note) => (
                <TrashRow note={note} onRestore={props.onRestore} onDelete={props.onDeleteForever} />
              )}
            </VirtualList>
          )
        ) : (
          <VirtualList
            scrollRef={listRef}
            items={items}
            getKey={(item) => ("heading" in item ? `heading:${item.heading}` : item.note.id)}
            estimateHeight={(item) => ("heading" in item ? HEADING_HEIGHT : ROW_HEIGHT)}
          >
            {(item) =>
              "heading" in item ? (
                <DateHeading label={item.heading} />
              ) : (
                <NoteRow
                  note={item.note}
                  selected={props.currentId === item.note.id}
                  onOpen={props.onOpen}
                  onTrash={props.onTrash}
                />
              )
            }
          </VirtualList>
        )}
      </div>
    </div>
  );
}
