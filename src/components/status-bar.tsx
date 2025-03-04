import { useState, useEffect } from "preact/hooks";

export function StatusBar(props: { text: string }) {
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
    <div
      style={{
        position: "absolute",
        bottom: 0,
        left: 0,
        right: 0,
        height: "20px",
        backgroundColor: "rgba(255, 255, 255, 0.1)",
        backdropFilter: "blur(10px)",
        WebkitBackdropFilter: "blur(10px)",
        padding: "0 10px",
        display: "flex",
        fontSize: "12px",
        gap: "10px",
        userSelect: "none",
        WebkitUserSelect: "none",
        MozUserSelect: "none",
        msUserSelect: "none",
      }}
    >
      <p>C: {props.text.length}</p>
      <p>W: {props.text.split(/\s+/).length}</p>
      {selectedText && (
        <p>
          Selected: C {selectedText.length} | {selectedRows}{" "}
          {selectedRows === 1 ? "row" : "rows"}
        </p>
      )}
    </div>
  );
}
