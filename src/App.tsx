import { invoke } from "@tauri-apps/api";
import { createId } from "@paralleldrive/cuid2";
import { useEffect, useState } from "preact/compat";
import type { JSX } from "preact/jsx-runtime";
import { useDebounce, useKeyPress } from "./hooks";
import "./App.css";

type Note = {
  id: string;
  created_at: string;
};

function App() {
  const [id, setId] = useState(createId());
  const [text, setText] = useState("");
  const { debouncedValue, setDebouncedValue } = useDebounce(text, 500);
  const [notes, setNotes] = useState<Note[]>([]);
  const [showHistory, setShowHistory] = useState(false);

  function handleOnClick() {
    invoke("create_note", {
      title: "Test",
      body: "Test",
    }).then((res) => {
      console.log(res);
    });

    invoke("get_notes").then((res) => {
      console.log(res);
    });
  }

  useKeyPress(["b"], (event) => {
    console.log("BIG CHUNGUSÆJ", event.key);
    setShowHistory((prev) => !prev);
  });
  useKeyPress(["s"], (event) => {
    console.log("Saving", event.key);
    setDebouncedValue(text);
  });

  // useEffect(() => {
  //   if (text.length === 0) {
  //     return;
  //   }

  //   invoke("add_user_command", {
  //     id: id,
  //     text: debouncedValue,
  //   }).then((res) => {
  //     console.log(res);
  //   });
  // }, [debouncedValue]);

  // useEffect(() => {
  //   invoke("get_notes").then((res) => {
  //     console.log(res);
  //   });
  // }, [debouncedValue]);

  // function handleOnChange(e: { target: { value: string } }) {
  //   setText(e.target.value);
  // }

  // function handleNoteChange(id: string) {
  //   setId(id);
  //   invoke("get_note_text", { id }).then((res) => {
  //     handleOnChange({ target: { value: res as string } });
  //     setText(res as string);
  //   });
  // }

  return (
    <div
      style={{
        width: "100%",
        border: "none",
        height: "calc(100dvh - 2em)",
        outline: "none",
        display: "flex",
      }}
    >
      <textarea
        placeholder="Jot down notes.."
        // onInput={
        //   handleOnChange as unknown as JSX.GenericEventHandler<HTMLTextAreaElement>
        // }
        onClick={handleOnClick}
        autoComplete={"off"}
        // value={text}
        style={{
          fontSize: "1em",
          minHeight: "100%",
          width: "100%",
          margin: "0px",
          padding: "1em",
          outline: "none",
          border: "none",
          resize: "none",
        }}
      />
      {/* {showHistory && (
        <div style={{ userSelect: "none" }}>
          {notes.map((n) => (
            <div
              style={{ padding: "1em" }}
              key={n.id}
              onClick={() => handleNoteChange(n.id)}
            >
              <p class={"shh"}>{n.id}</p>
            </div>
          ))}
        </div>
      )} */}
    </div>
  );
}

export default App;
