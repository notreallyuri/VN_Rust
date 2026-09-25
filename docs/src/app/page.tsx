export default function Home() {
  return (
    <main className="mx-auto flex min-h-screen max-w-2xl flex-col justify-center gap-4 px-6">
      <h1 className="font-semibold text-3xl tracking-tight">VN_Rust</h1>
      <p className="text-base/7 opacity-70">
        A visual novel engine in Rust: a story language with a compiler behind
        it, and a raylib engine that plays what it compiles.
      </p>
      <p className="text-sm opacity-50">
        The guide lives in the repository&rsquo;s README files while this site
        is built. See TODO.md, milestone 11, for what goes where.
      </p>
    </main>
  );
}
