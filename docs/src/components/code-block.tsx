export function CodeBlock({
  name,
  lines,
  numbered = false,
}: {
  name: string;
  lines: string[];
  numbered?: boolean;
}) {
  return (
    <figure className="overflow-hidden rounded-lg border border-line bg-faint">
      <figcaption className="border-line border-b px-4 py-2.5 font-mono text-[0.7rem] text-muted uppercase tracking-widest">
        {name}
      </figcaption>
      <pre className="overflow-x-auto px-4 py-4 font-mono text-[0.82rem] leading-6">
        <code>
          {lines.map((line, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
            <span key={index} className="grid grid-cols-[2ch_1fr] gap-4">
              {numbered ? (
                <span className="select-none text-right text-muted/50">
                  {index + 1}
                </span>
              ) : (
                <span />
              )}
              <span className="whitespace-pre">{line || " "}</span>
            </span>
          ))}
        </code>
      </pre>
    </figure>
  );
}
