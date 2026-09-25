export type NavItem = { title: string; href: string; summary?: string };
export type NavSection = { title: string; items: NavItem[] };

export const nav: NavSection[] = [
  {
    title: "Start here",
    items: [
      {
        title: "What novn is",
        href: "/docs",
        summary: "The three layers, and which crate is which.",
      },
      {
        title: "Your first novel",
        href: "/docs/first-novel",
        summary: "From novn new to a scene on screen.",
      },
    ],
  },
  {
    title: "The story language",
    items: [
      {
        title: "Scenes and characters",
        href: "/docs/scenes",
        summary: "Scenes, showing characters, backgrounds, transitions, audio.",
      },
      {
        title: "Dialogue and narration",
        href: "/docs/dialogue",
        summary: "Speakers, narration, variables in text, tags.",
      },
      {
        title: "Choices and jumps",
        href: "/docs/choices",
        summary: "Branching, gated options, pictures, jump.",
      },
    ],
  },
];
