import { invoke } from "@tauri-apps/api";
import { useEffect, useRef, useState } from "preact/compat";
import type { JSX } from "preact/jsx-runtime";
import "./App.css";
import { useDebounce, useKeyPress } from "./hooks";
import { IconCloseCircle } from "./icons/IconCloseCircle";
import { getRelativeDate } from "./utils";

type Note = {
  id: number;
  title: string;
  created_at: string;
};

function getSubstring(str: string, length: number): string {
  // Return a substring from the start of the string to the specified length
  return str.substring(0, length);
}

function App() {
  const [id, setId] = useState<number>();
  const [text, setText] = useState("");
  const { debouncedValue, setDebouncedValue } = useDebounce(text, 300);
  const [notes, setNotes] = useState<Note[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useKeyPress(["b"], () => {
    setShowHistory((prev) => !prev);
  });

  useKeyPress(["s"], () => {
    setDebouncedValue(text);
    void save();
  });

  useKeyPress(["t"], async () => {
    await save();
    setId(undefined);
    setText("");
  });

  async function save() {
    if (text.length === 0 && !Boolean(id)) {
      return;
    }

    const newId = await invoke("create_note", {
      title: getSubstring(text, 70),
      body: text,
      id,
    });

    setId(newId as number);
  }

  useEffect(() => {
    save();
  }, [debouncedValue]);

  useEffect(() => {
    invoke("get_notes").then((res) => {
      console.log(res);
      setNotes(res as Note[]);
    });
  }, [debouncedValue]);

  function handleOnChange(e: JSX.TargetedInputEvent<HTMLTextAreaElement>) {
    if (!e) {
      return;
    }

    setText(e.currentTarget?.value ?? "");
  }

  const handleOnKeyDown: JSX.KeyboardEventHandler<HTMLTextAreaElement> = (
    e
  ) => {
    if (e.key === "Tab") {
      e.preventDefault();
      const textarea = textareaRef.current;
      if (!textarea) {
        return;
      }

      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;

      // Insert a tab character at the cursor position
      const newValue = text.substring(0, start) + "\t" + text.substring(end);
      setText(newValue);

      // Update the cursor position after the value is set
      textarea.selectionStart = textarea.selectionEnd = start + 1;
      return;
    }
  };

  async function handleNoteChange(id: number | undefined) {
    setId(id);

    if (id === undefined) {
      setText("");
    } else {
      const newText: string = await invoke("get_note_text", { id });
      setText(newText);
    }

    const notes: Note[] = await invoke("get_notes");
    setNotes(notes);
  }

  return (
    <div
      style={{
        width: "100%",
        border: "none",
        outline: "none",
        display: "flex",
        overflow: "none",
        flex: 1,
        height: "100%",
      }}
    >
      <textarea
        placeholder="⌘ + b to show history"
        onInput={
          handleOnChange as unknown as JSX.InputEventHandler<HTMLTextAreaElement>
        }
        onKeyDown={handleOnKeyDown}
        id="note-text-area"
        autoComplete={"off"}
        autoFocus={true}
        value={text}
        style={{
          fontSize: "1em",
          minHeight: "100%",
          width: "100%",
          margin: "0px",
          padding: "1em",
          outline: "none",
          border: "none",
          // border: "1px solid #d6d6d6a7",
          resize: "none",
          boxSizing: "border-box",
        }}
        ref={textareaRef}
      />
      {showHistory && (
        <div id="history-container">
          <div id="history-content-container">
            <div class="air-element" />
            {notes.map((n) => (
              <NoteCard
                id={n.id}
                title={n.title}
                created_at={n.created_at}
                handleNoteChange={handleNoteChange}
                selected={id === n.id}
              />
            ))}
            <div class="air-element" />
          </div>
        </div>
      )}
    </div>
  );
}

function NoteCard(props: {
  id: number;
  title: string;
  created_at: string;
  handleNoteChange: (id: number | undefined) => void;
  selected?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const button = ref.current;

  useEffect(() => {
    if (!button) {
      return;
    }

    // Add keydown event
    button.addEventListener("mousedown", function () {
      button.classList.add("button-pressed");
    });

    // Remove the class when the key is released
    button.addEventListener("mouseup", function () {
      button.classList.remove("button-pressed");
    });

    return () => {
      button.removeEventListener("mousedown", function () {
        button.classList.remove("button-pressed");
      });

      button.removeEventListener("mouseup", function () {
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
        class={"note-card shh" + (props.selected ? " selected" : "")}
        ref={ref}
        key={props.id}
        onClick={() => props.handleNoteChange(props.id)}
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

export default App;
