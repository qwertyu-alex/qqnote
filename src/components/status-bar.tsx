import { useEffect, useState } from "preact/hooks";
import { caretPosition, countCharacters, countWords } from "../utils";

const numberFormat = new Intl.NumberFormat();

function count(n: number, noun: string): string {
  return `${numberFormat.format(n)} ${noun}${n === 1 ? "" : "s"}`;
}

type Caret = { line: number; col: number; selectedWords: number; selectedChars: number } | null;

export function StatusBar(props: {
  textarea: () => HTMLTextAreaElement | null;
  chars: number;
  words: number;
  status: string | null;
  error: boolean;
}) {
  const [caret, setCaret] = useState<Caret>(null);

  useEffect(() => {
    let timer: number | undefined;
    const update = () => {
      timer = undefined;
      const textarea = props.textarea();
      if (!textarea) {
        return;
      }
      const { selectionStart: start, selectionEnd: end, selectionDirection, value } = textarea;
      const { line, col } = caretPosition(value, selectionDirection === "backward" ? start : end);
      const selected = start === end ? "" : value.slice(start, end);
      setCaret({
        line,
        col,
        selectedWords: countWords(selected),
        selectedChars: countCharacters(selected),
      });
    };
    // Fires for typing, arrow keys, clicks and note switches; coalesce bursts into one update.
    const schedule = () => {
      timer ??= window.setTimeout(update, 0);
    };

    update();
    document.addEventListener("selectionchange", schedule);
    return () => {
      document.removeEventListener("selectionchange", schedule);
      window.clearTimeout(timer);
    };
  }, []);

  const selecting = caret !== null && caret.selectedChars > 0;

  return (
    <div class="status-bar">
      <span>
        Ln {caret?.line ?? 1}, Col {caret?.col ?? 1}
      </span>
      <span class="status-separator" />
      <span>
        {selecting
          ? `${numberFormat.format(caret.selectedWords)} of ${count(props.words, "word")}`
          : count(props.words, "word")}
      </span>
      <span class="status-separator" />
      <span>
        {selecting
          ? `${numberFormat.format(caret.selectedChars)} of ${count(props.chars, "character")}`
          : count(props.chars, "character")}
      </span>
      {props.status && (
        <span class={`status-end${props.error ? " error" : ""}`}>{props.status}</span>
      )}
    </div>
  );
}
