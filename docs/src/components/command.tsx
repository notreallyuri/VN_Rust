import { highlight } from "@/highlight";

export function Command({ children }: { children: string }) {
  const [tokens] = highlight(children, "sh");
  return (
    <code className="inline-flex items-center gap-2.5 rounded-md border border-line bg-faint px-3 py-2 font-mono text-[0.82rem]">
      <span aria-hidden className="select-none text-accent/70">
        $
      </span>
      <span>
        {tokens.map((token, at) => (
          <span
            // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
            key={at}
            className={token.kind ? `tok-${token.kind}` : undefined}
          >
            {token.text}
          </span>
        ))}
      </span>
    </code>
  );
}
