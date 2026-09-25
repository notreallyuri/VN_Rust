import { CopyButton } from "@/components/copy-button";
import { Frame } from "@/components/frame";
import { highlight, type Token } from "@/highlight";

const SHELLS = new Set(["sh", "bash", "shell", "console"]);

export function CodeBlock({
  code,
  name,
  language,
  numbered = false,
}: {
  code: string;
  name?: string;
  language?: string;
  numbered?: boolean;
}) {
  const body = code.replace(/\n$/, "");
  const lines: Token[][] = highlight(body, language);
  const shell = language ? SHELLS.has(language) : false;
  const gutter = numbered || shell;

  return (
    <Frame action={<CopyButton text={body} />} name={name}>
      <pre className="overflow-x-auto px-4 py-4 font-mono text-[0.82rem] leading-6">
        <code>
          {lines.map((tokens, index) => (
            <span
              // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
              key={index}
              className={gutter ? "grid grid-cols-[2ch_1fr] gap-3" : "block"}
            >
              {gutter ? (
                <span
                  aria-hidden={shell}
                  className={
                    shell
                      ? "select-none text-accent/70"
                      : "select-none text-right text-muted/50"
                  }
                >
                  {shell ? (tokens.length > 0 ? "$" : "") : index + 1}
                </span>
              ) : null}
              <span className="whitespace-pre">
                {tokens.length === 0
                  ? " "
                  : tokens.map((token, at) => (
                      <span
                        // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
                        key={at}
                        className={token.kind ? `tok-${token.kind}` : undefined}
                      >
                        {token.text}
                      </span>
                    ))}
              </span>
            </span>
          ))}
        </code>
      </pre>
    </Frame>
  );
}

export function fenceFrom(children: unknown): {
  code: string;
  language?: string;
} {
  const node = children as
    | { props?: { children?: unknown; className?: string } }
    | undefined;
  const inner = node?.props?.children;
  const className = node?.props?.className ?? "";
  const match = /language-(\w+)/.exec(className);
  return {
    code: typeof inner === "string" ? inner : "",
    language: match?.[1],
  };
}
