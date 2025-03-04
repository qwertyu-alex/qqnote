import { invoke } from "@tauri-apps/api";
import { useEffect, useRef, useState } from "preact/compat";
import type { JSX } from "preact/jsx-runtime";
import "./App.css";
import { useDebounce, useKeyPress } from "./hooks";
import { NoteCard } from "./components/notecard";

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
    if (text.length === 0 && !id) {
      return;
    }

    const newId = await invoke("create_note", {
      title: getSubstring(text, 70),
      body: text,
      id,
    });

    setId(newId as number);
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
  useEffect(() => {
    save();
  }, [debouncedValue]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
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
      const newValue = `${text.substring(0, start)}\t${text.substring(end)}`;
      setText(newValue);

      // Use setTimeout to ensure the DOM has updated
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 1;
      }, 0);
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
      <textarea
        placeholder="⌘ + b to show history"
        onInput={
          handleOnChange as unknown as JSX.InputEventHandler<HTMLTextAreaElement>
        }
        onKeyDown={handleOnKeyDown}
        id="note-text-area"
        autoComplete={"off"}
        // biome-ignore lint/a11y/noAutofocus: <explanation>
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
          boxSizing: "border-box",
          backgroundColor: "transparent",
          borderRadius: "var(--border-radius)",
        }}
        ref={textareaRef}
      />
      {showHistory && (
        <div id="history-container">
          <div id="history-content-container">
            <div class="air-element" />
            {notes.map((n) => (
              <NoteCard
                key={n.id}
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

export default App;
