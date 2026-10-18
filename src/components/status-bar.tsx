import { useState, useEffect } from "preact/hooks";

export function StatusBar(props: { chars: number; words: number; notice: string | null }) {
  const [selectedText, setSelectedText] = useState<string>("");
  const [selectedRows, setSelectedRows] = useState<number>(0);

  useEffect(() => {
    const handleSelectionChange = () => {
      const selection = window.getSelection();
      if (selection && selection.toString()) {
        // Get selected text
        const text = selection.toString();
        setSelectedText(text);

        // Count number of rows in selection
        const rowCount = text.split("\n").length;
        setSelectedRows(rowCount);
      } else {
        setSelectedText("");
        setSelectedRows(0);
      }
    };

    document.addEventListener("selectionchange", handleSelectionChange);
    return () => {
      document.removeEventListener("selectionchange", handleSelectionChange);
    };
  }, []);

  return (
    <div class="status-bar">
      <p>C: {props.chars}</p>
      <p>W: {props.words}</p>
      {selectedText && (
        <p>
          Selected: C {selectedText.length} | {selectedRows}{" "}
          {selectedRows === 1 ? "row" : "rows"}
        </p>
      )}
      {props.notice && <p style={{ marginLeft: "auto" }}>{props.notice}</p>}
    </div>
  );
}
