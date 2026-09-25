import { CodeBlock } from "@/components/code-block";

const story = [
  "scene archive_start:",
  "  background archive_office with fade",
  "  show registrar netural at right with dissolve",
  '  registrar "You’re the new one. Sit."',
  "  jump vault",
];

const diagnostics = [
  {
    where: "01.story:3: error:",
    message:
      "`show registrar netural`: 'registrar' has no image 'netural' (images: neutral, stern);",
    suggestion: "did you mean 'neutral'?",
  },
  {
    where: "01.story:5: error:",
    message: "`jump vault`: no scene with that name",
    suggestion: null,
  },
];

const pillars = [
  {
    title: "A language, not a config file",
    body: "Scenes, dialogue, choices and conditions in a syntax a writer can hold in their head. It has a lexer, a parser, a compiler and a VM behind it, and no embedded scripting language: a story describes flow and asks the engine for things, it never owns logic.",
  },
  {
    title: "An engine that is already furnished",
    body: "Menus, saves with migrations and thumbnails, rollback, settings, localisation, transitions, keyboard and gamepad navigation, hot reload while the game runs. Configure the screens it ships or replace any of them with your own.",
  },
  {
    title: "Tools that answer before you run",
    body: "vn check reads the whole story against the game's schema. vn new starts a project, vn fmt tidies a script, vn lsp gives any editor completion and diagnostics as you type.",
  },
];

export default function Home() {
  return (
    <div className="mx-auto w-full max-w-5xl px-6">
      <header className="flex items-center justify-between py-6">
        <span className="font-mono text-sm tracking-tight">VN_Rust</span>
        <nav className="flex items-center gap-6 text-muted text-sm">
          <a
            className="transition-colors hover:text-foreground"
            href="https://github.com/notreallyuri/VN_Rust"
          >
            GitHub
          </a>
        </nav>
      </header>

      <main>
        <section className="border-line border-b py-20 sm:py-28">
          <h1 className="max-w-3xl text-balance font-semibold text-4xl leading-[1.1] tracking-tight sm:text-6xl">
            A visual novel engine that reads the story before it runs it.
          </h1>
          <p className="mt-7 max-w-2xl text-base/7 text-muted sm:text-lg/8">
            VN_Rust compiles your script, checks every character, image,
            variable and jump against the game&rsquo;s own schema, and plays
            what it compiled on a raylib engine. Written in Rust, for visual
            novels that outgrow a folder of scripts.
          </p>
          <div className="mt-10 flex flex-wrap items-center gap-x-8 gap-y-4">
            <a
              className="rounded-md bg-foreground px-5 py-2.5 font-medium text-background text-sm transition-opacity hover:opacity-85"
              href="https://github.com/notreallyuri/VN_Rust#readme"
            >
              Read the guide
            </a>
            <span className="font-mono text-muted text-sm">
              cargo run -p vn_cli -- new my-novel
            </span>
          </div>
        </section>

        <section className="border-line border-b py-20">
          <h2 className="max-w-2xl text-balance font-semibold text-2xl tracking-tight sm:text-3xl">
            A typo is a compile error, not a black screen
          </h2>
          <p className="mt-4 max-w-2xl text-base/7 text-muted">
            Every name a story mentions has to exist: the characters, their
            images, the variables a condition reads, the scene a jump lands in.
            The checker reads the whole project in milliseconds and says what it
            expected instead.
          </p>
          <div className="mt-10 grid items-start gap-6 lg:grid-cols-2">
            <CodeBlock name="story/01.story" lines={story} numbered />
            <figure className="overflow-hidden rounded-lg border border-line bg-faint">
              <figcaption className="border-line border-b px-4 py-2.5 font-mono text-[0.7rem] text-muted uppercase tracking-widest">
                vn check assets
              </figcaption>
              <div className="space-y-3 px-4 py-4 font-mono text-[0.78rem] leading-6">
                {diagnostics.map((line) => (
                  <p key={line.where}>
                    <span className="text-muted">{line.where} </span>
                    {line.message}
                    {line.suggestion ? (
                      <span className="text-accent"> {line.suggestion}</span>
                    ) : null}
                  </p>
                ))}
                <p className="text-muted">
                  1 file, 1 scene checked against schema.json: 2 errors, 0
                  warnings
                </p>
              </div>
            </figure>
          </div>
        </section>

        <section className="grid gap-10 border-line border-b py-20 sm:grid-cols-3 sm:gap-8">
          {pillars.map((pillar) => (
            <div key={pillar.title}>
              <h3 className="font-medium text-base tracking-tight">
                {pillar.title}
              </h3>
              <p className="mt-3 text-muted text-sm/6">{pillar.body}</p>
            </div>
          ))}
        </section>
      </main>

      <footer className="flex flex-wrap items-center justify-between gap-4 py-10 text-muted text-sm">
        <span>
          The guide lives in the repository while this site is built &mdash; see
          milestone 11.
        </span>
        <span className="font-mono text-xs">MIT</span>
      </footer>
    </div>
  );
}
