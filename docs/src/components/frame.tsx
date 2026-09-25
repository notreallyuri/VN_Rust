import type { ReactNode } from "react";

export function Frame({
  name,
  action,
  children,
}: {
  name?: string;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <figure className="group relative my-6 overflow-hidden rounded-lg border border-line bg-faint">
      {name ? (
        <figcaption className="border-line border-b px-4 py-2.5 font-mono text-[0.7rem] text-muted uppercase tracking-widest">
          {name}
        </figcaption>
      ) : null}
      {action}
      {children}
    </figure>
  );
}
