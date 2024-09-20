import { invoke } from "@tauri-apps/api";
import { useEffect, useRef, useState } from "preact/compat";
import type { JSX } from "preact/jsx-runtime";
import "./App.css";
import { useDebounce, useKeyPress } from "./hooks";
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

  function handleOnChange(e: { target: { value: string } }) {
    setText(e.target.value);
  }

  function handleNoteChange(id: number) {
    setId(id);
    invoke("get_note_text", { id }).then((res) => {
      handleOnChange({ target: { value: res as string } });
      setText(res as string);
    });
  }

  return (
    <div
      style={{
        width: "100%",
        border: "none",
        outline: "none",
        display: "flex",
        overflow: "none",
      }}
    >
      <textarea
        placeholder="⌘ + b to show history"
        onInput={
          handleOnChange as unknown as JSX.GenericEventHandler<HTMLTextAreaElement>
        }
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
          resize: "none",
          maxHeight: "100svh",
        }}
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
  handleNoteChange: (id: number) => void;
  selected?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const button = ref.current;

  useEffect(() => {
    if (!button) {
      return;
    }

    // Add keydown event
    document.addEventListener("mousedown", function () {
      button.classList.add("button-pressed");
      console.log("mousedown");
    });

    // Remove the class when the key is released
    document.addEventListener("mouseup", function () {
      button.classList.remove("button-pressed");
    });

    return () => {
      document.removeEventListener("mousedown", function () {
        button?.classList.remove("button-pressed");
      });

      document.removeEventListener("mouseup", function () {
        button?.classList.remove("button-pressed");
      });
    };
  }, [ref, button]);

  return (
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
  );
}

export default App;
