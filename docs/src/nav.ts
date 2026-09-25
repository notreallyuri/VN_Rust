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
        title: "Writing a scene",
        href: "/docs/scenes",
        summary: "Dialogue, characters, backgrounds, transitions.",
      },
    ],
  },
];

export const flat = nav.flatMap((section) => section.items);
