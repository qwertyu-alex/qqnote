// Sample notes, the title/preview split, date formats and the window frame for the mockups.
// "Now" is Monday 19 October 2026, 10:24.

const NOW = new Date(2026, 9, 19, 10, 24);
const oct = (day, h, m) => new Date(2026, 9, day, h, m);
const sep = (day, h, m) => new Date(2026, 8, day, h, m);

const NOTES = [
  {
    updated: oct(19, 10, 22),
    body: "Monday standup\n\n✅ Shipped the Caps Lock launcher\n✅ Dark mode, and a dark Dock icon to match\n⏳ README with screenshots\n❓ Ask Jonas about importing from Apple Notes\n\nNext demo: Thursday 14:00, room 3B",
  },
  { updated: oct(19, 9, 10), body: "Groceries: oat milk, eggs, coffee beans, lemons, basil, parmesan, bread" },
  { updated: oct(18, 21, 39), body: "Cabin wifi → network Fjord-Guest, password salmon-lantern-42" },
  { updated: oct(18, 18, 5), body: "Gift ideas for Sam\n- film camera\n- that ramen cookbook\n- climbing day pass" },
  { updated: oct(17, 19, 30), body: "Pasta al limone\n200 g spaghetti, 1 lemon (zest + juice), 50 g butter, parmesan" },
  { updated: oct(15, 8, 12), body: "Book club: The Left Hand of Darkness, chapters 1–6 by Thursday" },
  { updated: oct(14, 16, 45), body: "Flight LH 2453 MUC → CPH\nFri 07:40, seat 14A, booking ref QX7P2L" },
  { updated: oct(13, 23, 2), body: "Ideas for the launch video\n- open on a Caps Lock close-up\n- the note appears instantly\n- cut to dark mode" },
  { updated: oct(9, 11, 30), body: "https://developer.apple.com/design/human-interface-guidelines/sidebars" },
  { updated: oct(2, 20, 15), body: "Rent split: me 1,150 / Sam 950, utilities 50/50, due on the 1st" },
  { updated: sep(28, 7, 50), body: "Dentist Tue Nov 4, 08:30, Dr. Lindqvist. Bring the insurance card" },
  { updated: sep(21, 22, 10), body: "Song stuck in my head: Pink + White, Frank Ocean" },
];

const TRASH = [
  { deleted: oct(17, 12, 4), body: "Old wifi password: lantern-salmon-41" },
  { deleted: oct(15, 9, 30), body: "Draft tweet: qqnote now follows your Mac's light and dark appearance" },
  { deleted: oct(3, 17, 45), body: "asdf" },
];

// Which rows are drawn as open (selected) and hovered.
const SELECTED = 0;
const HOVERED = 2;

/** The sidebar only gets the first 70 characters: the first line is the title, the rest a preview. */
function split(body) {
  const chars = Array.from(body);
  const cut = chars.slice(0, 70).join("");
  const more = chars.length > 70;
  const lines = cut.split("\n").map((l) => l.trim()).filter(Boolean);
  let title = lines[0] ?? "";
  let preview = lines
    .slice(1)
    .map((l) => l.replace(/^[-*•]\s+/, ""))
    .join("\u00a0· ");
  if (more) {
    if (preview) preview += "…";
    else title += "…";
  }
  return { title: title || "Empty note", preview, empty: !title };
}

function dayDiff(date) {
  const a = new Date(NOW.getFullYear(), NOW.getMonth(), NOW.getDate());
  const b = new Date(date.getFullYear(), date.getMonth(), date.getDate());
  return Math.round((a - b) / 86400000);
}

const hhmm = (d) => `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
const weekday = (d) => d.toLocaleString("en-US", { weekday: "long" });
const monthDay = (d) => d.toLocaleString("en-US", { month: "short", day: "numeric" });

/** Today, Yesterday, Previous 7 Days, then month names. */
function groupOf(d) {
  const days = dayDiff(d);
  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return "Previous 7 Days";
  return d.toLocaleString("en-US", { month: "long" });
}

/** Short time for a row that sits under a date group: 10:22, Saturday, Oct 9. */
function shortTime(d) {
  const days = dayDiff(d);
  if (days <= 1) return hhmm(d);
  if (days < 7) return weekday(d);
  return monthDay(d);
}

/** Self-contained time for a row without groups: 2 min ago, Yesterday 21:39, Saturday, Oct 9. */
function relativeTime(d) {
  const mins = Math.round((NOW - d) / 60000);
  const days = dayDiff(d);
  if (mins < 60) return `${mins} min ago`;
  if (days === 0) return `Today ${hhmm(d)}`;
  if (days === 1) return `Yesterday ${hhmm(d)}`;
  if (days < 7) return weekday(d);
  return monthDay(d);
}

function deletedTime(d) {
  const days = dayDiff(d);
  if (days === 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 7) return weekday(d);
  return monthDay(d);
}

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

const ICONS = {
  trash: (s = 14) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><path d="M2.75 4.25h10.5M6.25 4.25v-1.5h3.5v1.5M4.25 4.25l.55 8.3a1.5 1.5 0 0 0 1.5 1.4h3.4a1.5 1.5 0 0 0 1.5-1.4l.55-8.3M6.75 7v4.25M9.25 7v4.25" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  search: (s = 13) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><circle cx="7" cy="7" r="4.6" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="m10.5 10.5 3.3 3.3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>`,
  restore: (s = 14) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><path d="M3.5 6.5h6.25a3.25 3.25 0 0 1 0 6.5H7M3.5 6.5 6 4M3.5 6.5 6 9" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  back: (s = 12) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><path d="M10 3 5 8l5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  x: (s = 12) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><path d="m4.5 4.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>`,
  compose: (s = 14) =>
    `<svg viewBox="0 0 16 16" width="${s}" height="${s}"><path d="M8 2.75H4.25a1.5 1.5 0 0 0-1.5 1.5v7.5a1.5 1.5 0 0 0 1.5 1.5h7.5a1.5 1.5 0 0 0 1.5-1.5V8M11.6 2.4a1.27 1.27 0 0 1 1.8 1.8L8.2 9.4l-2.4.6.6-2.4z" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
};

/** Notes with their split text and dates, grouped when `grouped` is set. */
function noteRows() {
  return NOTES.map((n, i) => ({
    ...split(n.body),
    date: n.updated,
    selected: i === SELECTED,
    hovered: i === HOVERED,
    group: groupOf(n.updated),
  }));
}

function trashRows() {
  return TRASH.map((n, i) => ({ ...split(n.body), date: n.deleted, hovered: i === 0 }));
}

/** Renders rows with a group label before each new group. */
function withGroups(rows, groupHtml, rowHtml) {
  let last = null;
  return rows
    .map((r) => {
      const label = r.group !== last ? groupHtml(r.group) : "";
      last = r.group;
      return label + rowHtml(r);
    })
    .join("");
}

function windowHtml(theme, sidebar) {
  const body = esc(NOTES[SELECTED].body);
  return `<div class="window theme-${theme}">
    <div class="titlebar"><div class="lights"><i></i><i></i><i></i></div>qqnote - ⌘ + t to create a new note</div>
    <div class="content">
      <div class="editor-col">
        <div class="editor">${body}<span class="caret"></span></div>
        <div class="status-bar"><span>Ln 8, Col 35</span><span class="sep"></span><span>36 words</span><span class="sep"></span><span>198 characters</span><span class="end">Saved</span></div>
      </div>
      <aside class="sidebar">${sidebar}</aside>
    </div>
  </div>`;
}

function panelHtml(theme, sidebar) {
  return `<div class="panel theme-${theme}" style="width: var(--sidebar-w)">
    <div class="titlebar"></div>
    <div class="content"><aside class="sidebar">${sidebar}</aside></div>
  </div>`;
}

/** variant: { id, name, tagline, sidebar(view, theme) } */
function renderBoard(variant) {
  document.body.className = `board ${variant.id}`;
  document.body.innerHTML = `
    <div class="board-head">
      <div class="board-title">${variant.name}</div>
      <div class="board-sub">${variant.tagline}</div>
    </div>
    <div class="frames">
      <figure><figcaption>Light · open note selected, “Cabin wifi” hovered</figcaption>${windowHtml("light", variant.sidebar("notes", "light"))}</figure>
      <figure><figcaption>Dark</figcaption>${windowHtml("dark", variant.sidebar("notes", "dark"))}</figure>
      <figure><figcaption>Trash</figcaption>${panelHtml("light", variant.sidebar("trash", "light"))}</figure>
    </div>`;
}
