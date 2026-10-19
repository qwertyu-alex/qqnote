/** Timestamps from the backend are UTC without a zone, e.g. "2026-10-19T10:22:00.123". */
export function parseTimestamp(timestamp: string): Date {
  return new Date(`${timestamp}Z`);
}

/** Calendar days between `date` and `now`: 0 for today, 1 for yesterday. */
function daysAgo(date: Date, now: Date): number {
  const day = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  return Math.round((today.getTime() - day.getTime()) / 86_400_000);
}

const timeOfDay = (date: Date) =>
  `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;

function monthDay(date: Date, now: Date): string {
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: date.getFullYear() === now.getFullYear() ? undefined : "numeric",
  });
}

/** The heading a note is listed under: Today, Yesterday, Previous 7 Days, a month this year, or a year. */
export function dateGroup(date: Date, now = new Date()): string {
  const days = daysAgo(date, now);
  if (days <= 0) {
    return "Today";
  }
  if (days === 1) {
    return "Yesterday";
  }
  if (days < 7) {
    return "Previous 7 Days";
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString("en-US", { month: "long" });
  }
  return String(date.getFullYear());
}

/** When a note was edited, short enough to sit under its heading: 10:22, Saturday, Oct 9. */
export function shortDate(date: Date, now = new Date()): string {
  const days = daysAgo(date, now);
  if (days <= 1) {
    return timeOfDay(date);
  }
  if (days < 7) {
    return date.toLocaleDateString("en-US", { weekday: "long" });
  }
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

/** For "Deleted …": today, yesterday, Saturday, Oct 3, Oct 3, 2025. */
export function deletedDate(date: Date, now = new Date()): string {
  const days = daysAgo(date, now);
  if (days <= 0) {
    return "today";
  }
  if (days === 1) {
    return "yesterday";
  }
  if (days < 7) {
    return date.toLocaleDateString("en-US", { weekday: "long" });
  }
  return monthDay(date, now);
}

/** Counts characters the way people do: an emoji is one character, not two UTF-16 units. */
export function countCharacters(text: string): number {
  return text.length - (text.match(/[\uD800-\uDBFF][\uDC00-\uDFFF]/g)?.length ?? 0);
}

export function countWords(text: string): number {
  const trimmed = text.trim();
  return trimmed === "" ? 0 : trimmed.split(/\s+/).length;
}

/** 1-based line and column of a position in the text. */
export function caretPosition(text: string, offset: number): { line: number; col: number } {
  let line = 1;
  let lineStart = 0;
  for (let i = text.indexOf("\n"); i !== -1 && i < offset; i = text.indexOf("\n", i + 1)) {
    line++;
    lineStart = i + 1;
  }
  return { line, col: countCharacters(text.slice(lineStart, offset)) + 1 };
}
