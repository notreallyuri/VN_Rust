import { Frame } from "@/components/frame";

export function CodeBlock({
  code,
  name,
  numbered = false,
}: {
  code: string;
  name?: string;
  numbered?: boolean;
}) {
  const lines = code.replace(/\n$/, "").split("\n");
  return (
    <Frame name={name}>
      <pre className="overflow-x-auto px-4 py-4 font-mono text-[0.82rem] leading-6">
        <code>
          {lines.map((line, index) => (
            <span
              // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
              key={index}
              className={numbered ? "grid grid-cols-[2ch_1fr] gap-4" : "block"}
            >
              {numbered ? (
                <span className="select-none text-right text-muted/50">
                  {index + 1}
                </span>
              ) : null}
              <span className="whitespace-pre">{line || " "}</span>
            </span>
          ))}
        </code>
      </pre>
    </Frame>
  );
}

export function codeFromPre(children: unknown): string {
  const node = children as { props?: { children?: unknown } } | undefined;
  const inner = node?.props?.children;
  return typeof inner === "string" ? inner : "";
}
