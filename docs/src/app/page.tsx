import Image from "next/image";
import Link from "next/link";
import { CodeBlock } from "@/components/code-block";
import { Command } from "@/components/command";
import { Frame } from "@/components/frame";
import { SiteHeader } from "@/components/site-header";
import playing from "@/screens/playing.webp";

const story = `scene archive_start:
  background archive_office with fade
  show registrar netural at right with dissolve
  registrar "You\u2019re the new one. Sit."
  jump vault`;

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
    title: "The language",
    body: "Scenes, dialogue, choices and conditions, in a syntax you can keep in your head. There is a lexer, a parser, a compiler and a VM behind it. Nothing is evaluated at runtime: a story says what happens and asks the engine for things, it never holds logic itself.",
  },
  {
    title: "The engine",
    body: "Menus, saves with migrations and thumbnails, rollback, settings, translations, transitions, keyboard and gamepad navigation, video, and hot reload while the game is running. Use the screens it comes with, or swap in your own.",
  },
  {
    title: "The tools",
    body: "novn new starts a project, novn check reads the story against the game's schema, novn fmt tidies a script, and novn lsp gives any editor completion and diagnostics while you type.",
  },
];

function Eyebrow({ children }: { children: string }) {
  return (
    <p className="mb-4 font-mono text-[0.7rem] text-accent uppercase tracking-[0.18em]">
      {children}
    </p>
  );
}

export default function Home() {
  return (
    <div className="relative">
      <div className="-z-10 pointer-events-none absolute inset-x-0 top-0 h-[32rem] bg-glow" />
      <SiteHeader />
      <div className="mx-auto w-full max-w-5xl px-6">
        <main>
          <section className="py-16 sm:py-20">
            <h1 className="font-semibold text-4xl tracking-tight sm:text-5xl">
              What is novn?
            </h1>
            <div className="mt-6 max-w-2xl space-y-4 text-base/7 text-muted sm:text-lg/8">
              <p>
                A visual novel engine I&rsquo;m building in Rust. You write the
                story in a small language of its own: scenes, dialogue, choices,
                conditions. The engine compiles it, checks the names in it
                against your game, and plays what came out.
              </p>
              <p>
                Most of what a novel needs is already in there: menus, saves,
                rollback, settings, translations. You set them up from Rust
                instead of building them again.
              </p>
            </div>
            <div className="mt-9 flex flex-wrap items-center gap-x-8 gap-y-4">
              <Link
                className="rounded-md bg-foreground px-5 py-2.5 font-medium text-background text-sm transition-opacity hover:opacity-85"
                href="/docs"
              >
                Read the guide
              </Link>
              <Command>cargo install novn-cli && novn new my_story</Command>
            </div>

            <figure className="mt-14">
              <div className="overflow-hidden rounded-xl border border-line shadow-[0_1px_40px_-12px_rgba(0,0,0,0.45)]">
                <Image
                  src={playing}
                  alt="A scene from the example game: a character, a name plate, the dialogue box and the HUD."
                  priority
                  className="w-full"
                />
              </div>
              <figcaption className="mt-3 text-muted text-sm">
                <em>God Is Watching</em>, the example that ships with it.
                Everything there except the art is the engine: the name plate,
                the box, the HUD, the letterboxing.
              </figcaption>
            </figure>
          </section>

          <section className="border-line border-t py-20">
            <Eyebrow>vn check</Eyebrow>
            <h2 className="max-w-2xl text-balance font-semibold text-2xl tracking-tight sm:text-3xl">
              Everything gets checked before the game runs
            </h2>
            <p className="mt-4 max-w-2xl text-base/7 text-muted">
              Every name a story mentions has to exist somewhere: the
              characters, their images, the variables a condition reads, the
              scene a jump lands in. Run{" "}
              <code className="font-mono text-foreground/80">vn check</code> and
              it reads the whole project in milliseconds, then tells you what it
              expected instead.
            </p>
            <div className="mt-10 grid items-start gap-6 lg:grid-cols-2">
              <CodeBlock
                name="story/01.story"
                code={story}
                language="story"
                numbered
              />
              <Frame name="novn check assets">
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
              </Frame>
            </div>
          </section>

          <section className="border-line border-t py-20">
            <Eyebrow>What is in it</Eyebrow>
            <div className="grid gap-10 sm:grid-cols-3 sm:gap-8">
              {pillars.map((pillar) => (
                <div key={pillar.title}>
                  <h3 className="font-medium text-base tracking-tight">
                    {pillar.title}
                  </h3>
                  <p className="mt-3 text-muted text-sm/6">{pillar.body}</p>
                </div>
              ))}
            </div>
          </section>
        </main>

        <footer className="flex flex-wrap items-center justify-between gap-4 border-line border-t py-10 text-muted text-sm">
          <span>
            Still early: the guide covers a few pages so far, and grows from the
            READMEs.
          </span>
          <span className="font-mono text-xs">v0.1.0</span>
        </footer>
      </div>
    </div>
  );
}
