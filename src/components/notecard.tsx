import { invoke } from "@tauri-apps/api";
import { useEffect, useRef } from "preact/compat";
import { IconCloseCircle } from "../icons/IconCloseCircle";
import { getRelativeDate } from "../utils";

export function NoteCard(props: {
  id: number;
  title: string;
  created_at: string;
  handleNoteChange: (id: number | undefined) => void;
  selected?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const button = ref.current;

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    if (!button) {
      return;
    }

    // Add keydown event
    button.addEventListener("mousedown", () => {
      button.classList.add("button-pressed");
    });

    // Remove the class when the key is released
    button.addEventListener("mouseup", () => {
      button.classList.remove("button-pressed");
    });

    return () => {
      button.removeEventListener("mousedown", () => {
        button.classList.remove("button-pressed");
      });

      button.removeEventListener("mouseup", () => {
        button.classList.remove("button-pressed");
      });
    };
  }, [ref, button]);

  function handleDelete() {
    console.log("delete", props.id);

    invoke("delete_note", { id: props.id }).then((res) => {
      if (res) {
        props.handleNoteChange(undefined);
      }
    });
  }

  return (
    <div class={"note-card-container"}>
      <div
        class={`note-card shh${props.selected ? " selected" : ""}`}
        ref={ref}
        key={props.id}
        onClick={() => props.handleNoteChange(props.id)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            props.handleNoteChange(props.id);
          }
        }}
      >
        <div class="note-text">
          <div class="note-text">
            <p class={"shh note-title"}>{props.title}</p>
            <p class={"shh note-created-at"}>
              {getRelativeDate(new Date(`${props.created_at}Z`), 7)}
            </p>
          </div>
        </div>
      </div>
      <IconCloseCircle
        className={"close-button-icon shh"}
        onClick={handleDelete}
      />
    </div>
  );
}
