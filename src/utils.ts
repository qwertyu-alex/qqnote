export function getRelativeDate(ddate: Date, cutOffDays: number): string {
  const now = new Date();

  // Get time differences
  const timeDiffMs = now.getTime() - ddate.getTime(); // difference in milliseconds
  const timeDiffSecs = Math.floor(timeDiffMs / 1000); // difference in seconds
  const timeDiffMins = Math.floor(timeDiffSecs / 60); // difference in minutes
  const timeDiffHours = Math.floor(timeDiffMins / 60); // difference in hours
  const daysDiff = Math.floor(timeDiffHours / 24); // difference in days

  // If today, handle relative time
  if (daysDiff === 0) {
    if (timeDiffMins < 1) {
      // If less than one minute ago, return seconds
      return `${timeDiffSecs} second${timeDiffSecs !== 1 ? "s" : ""} ago`;
    } else if (timeDiffMins < 59) {
      // If within the last 10 minutes, return in minutes and seconds
      const seconds = timeDiffSecs % 60;
      return `${timeDiffMins} min${
        timeDiffMins !== 1 ? "s" : ""
      } and ${seconds} sec${seconds !== 1 ? "s" : ""} ago`;
    } else {
      // If today but not in the last 2 hours, return time in 24-hour format
      const hours = String(ddate.getHours()).padStart(2, "0");
      const minutes = String(ddate.getMinutes()).padStart(2, "0");
      return `Today at ${hours}:${minutes}`;
    }
  }

  // Include time for "Yesterday"
  if (daysDiff === 1) {
    const hours = String(ddate.getHours()).padStart(2, "0");
    const minutes = String(ddate.getMinutes()).padStart(2, "0");
    return `Yesterday at ${hours}:${minutes}`;
  }

  // Include time for dates within the cutoff days
  if (daysDiff > 0 && daysDiff <= cutOffDays) {
    const hours = String(ddate.getHours()).padStart(2, "0");
    const minutes = String(ddate.getMinutes()).padStart(2, "0");
    return `${daysDiff} day${
      daysDiff !== 1 ? "s" : ""
    } ago at ${hours}:${minutes}`;
  }

  // For dates outside the cutoff range, return in YYYY-MM-DD format
  const year = ddate.getFullYear();
  const month = String(ddate.getMonth() + 1).padStart(2, "0");
  const day = String(ddate.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
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
