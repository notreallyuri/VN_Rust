export type Section = {
  r: string;
  p: string;
  h: string;
  a: string;
  t: string;
};

export type Hit = Section & { score: number; excerpt: string };

const INDEX = `${process.env.NEXT_PUBLIC_BASE_PATH ?? ""}/search-index.json`;

let loading: Promise<Section[]> | null = null;

export function ready(): Promise<Section[]> {
  loading ??= fetch(INDEX).then((response) => {
    if (!response.ok) throw new Error(`${INDEX}: ${response.status}`);
    return response.json() as Promise<Section[]>;
  });
  return loading;
}

function terms(query: string): string[] {
  return query
    .toLowerCase()
    .split(/[^\w:.-]+/)
    .filter((term) => term.length > 1);
}

function count(haystack: string, term: string): number {
  let found = 0;
  let at = haystack.indexOf(term);
  while (at !== -1) {
    found += 1;
    at = haystack.indexOf(term, at + term.length);
  }
  return found;
}

const WIDTH = 150;
const BEFORE = 50;

function wordStart(text: string, at: number): number {
  if (at <= 0) return 0;
  const space = text.indexOf(" ", at);
  return space === -1 || space > at + 20 ? at : space + 1;
}

function wordEnd(text: string, at: number): number {
  if (at >= text.length) return text.length;
  const space = text.lastIndexOf(" ", at);
  return space <= 0 || space < at - 20 ? at : space;
}

function excerpt(text: string, term: string): string {
  const found = text.toLowerCase().indexOf(term);
  const from = wordStart(text, found === -1 ? 0 : Math.max(0, found - BEFORE));
  const to = wordEnd(text, Math.min(text.length, from + WIDTH));
  return (
    (from > 0 ? "…" : "") + text.slice(from, to) + (to < text.length ? "…" : "")
  );
}

export function parts(text: string, query: string): string[] {
  const wanted = terms(query);
  if (wanted.length === 0) return [text];
  const pattern = new RegExp(
    `(${wanted.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`,
    "gi",
  );
  return text.split(pattern);
}

export function search(sections: Section[], query: string, limit = 12): Hit[] {
  const wanted = terms(query);
  if (wanted.length === 0) return [];

  const hits: Hit[] = [];
  for (const section of sections) {
    const page = section.p.toLowerCase();
    const heading = section.h.toLowerCase();
    const body = section.t.toLowerCase();

    let score = 0;
    let missing = false;
    for (const term of wanted) {
      const inPage = count(page, term);
      const inHeading = count(heading, term);
      const inBody = count(body, term);
      if (inPage + inHeading + inBody === 0) {
        missing = true;
        break;
      }
      score += inPage * 12 + inHeading * 8 + Math.min(inBody, 6);
      if (page === term || heading === term) score += 10;
    }
    if (missing) continue;

    if (body.includes(query.toLowerCase())) score += 15;
    if (!section.h) score += 2;
    hits.push({ ...section, score, excerpt: excerpt(section.t, wanted[0]) });
  }

  return hits.sort((a, b) => b.score - a.score).slice(0, limit);
}
