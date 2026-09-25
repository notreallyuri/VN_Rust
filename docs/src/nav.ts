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
      {
        title: "Variables and conditions",
        href: "/docs/variables",
        summary: "set, add, if and else, and what a condition may hold.",
      },
      {
        title: "Engine commands",
        href: "/docs/commands",
        summary: "call, what is checked, and asking the player for a value.",
      },
      {
        title: "Rollback",
        href: "/docs/rollback",
        summary: "commit, final choices, points of no return.",
      },
      {
        title: "Identifiers and layout",
        href: "/docs/syntax",
        summary:
          "Names, indentation, errors, and what the language leaves out.",
      },
    ],
  },
  {
    title: "The engine",
    items: [
      {
        title: "Building an app",
        href: "/docs/engine/app",
        summary: "VnApp, the registries, the schema, where things live.",
      },
      {
        title: "The screens you get",
        href: "/docs/engine/screens",
        summary: "Start screen, main menu, actions, the playing screen.",
      },
      {
        title: "Menus, settings and overlays",
        href: "/docs/engine/menus",
        summary: "Pause menu, dialogs, settings, notifications, controls, log.",
      },
      {
        title: "Story integration",
        href: "/docs/engine/story",
        summary: "Commands, story words, hooks, game state, hot reload.",
      },
      {
        title: "Look and feel",
        href: "/docs/engine/look",
        summary: "Theme, corners, panels, buttons, layouts.",
      },
      {
        title: "The picture",
        href: "/docs/engine/picture",
        summary:
          "Render target, resolution, shaders, effects, transitions, scenery.",
      },
      {
        title: "Input",
        href: "/docs/engine/input",
        summary:
          "Keyboard and gamepad, prompts, image maps, dragging, tooltips.",
      },
      {
        title: "Audio and video",
        href: "/docs/engine/media",
        summary: "Music that follows the screen, voice, cutscenes, codecs.",
      },
      {
        title: "Languages",
        href: "/docs/engine/languages",
        summary: "Catalogues, the screens' own labels, what stays in ids.",
      },
      {
        title: "Screens of your own",
        href: "/docs/engine/custom-screens",
        summary: "Screens, overlays, text input, what the manager holds.",
      },
      {
        title: "Character visuals",
        href: "/docs/engine/visuals",
        summary: "The seam, puppets, parameters, lip sync, writing a backend.",
      },
      {
        title: "Saving and rollback",
        href: "/docs/engine/saving",
        summary: "Save files, autosave, thumbnails, rollback, migrations.",
      },
      {
        title: "Assets and fonts",
        href: "/docs/engine/assets",
        summary: "Folders or embedded, textures, placeholders, fonts.",
      },
    ],
  },
];
