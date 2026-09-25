import type { MDXComponents } from "mdx/types";
import Link from "next/link";
import type { ComponentPropsWithoutRef } from "react";
import { CodeBlock, fenceFrom } from "@/components/code-block";

function Anchor({ href = "", ...rest }: ComponentPropsWithoutRef<"a">) {
  const external = href.startsWith("http");
  const className =
    "text-accent underline decoration-accent/30 underline-offset-4 transition-colors hover:decoration-accent";
  return external ? (
    <a className={className} href={href} rel="noreferrer" {...rest} />
  ) : (
    <Link className={className} href={href} {...rest} />
  );
}

const components: MDXComponents = {
  h1: (props) => (
    <h1
      className="mt-2 mb-5 font-semibold text-3xl tracking-tight sm:text-4xl"
      {...props}
    />
  ),
  h2: (props) => (
    <h2
      className="mt-14 mb-4 scroll-mt-24 border-line border-t pt-10 font-semibold text-2xl tracking-tight"
      {...props}
    />
  ),
  h3: (props) => (
    <h3
      className="mt-10 mb-3 scroll-mt-24 font-medium text-lg tracking-tight"
      {...props}
    />
  ),
  p: (props) => <p className="my-4 text-base/7 text-muted" {...props} />,
  ul: (props) => (
    <ul
      className="my-4 list-disc space-y-2 pl-5 text-base/7 text-muted"
      {...props}
    />
  ),
  ol: (props) => (
    <ol
      className="my-4 list-decimal space-y-2 pl-5 text-base/7 text-muted"
      {...props}
    />
  ),
  li: (props) => <li className="pl-1" {...props} />,
  strong: (props) => (
    <strong className="font-medium text-foreground" {...props} />
  ),
  em: (props) => <em className="italic" {...props} />,
  a: Anchor,
  code: (props) => (
    <code
      className="rounded bg-faint px-1.5 py-0.5 font-mono text-[0.85em] text-foreground"
      {...props}
    />
  ),
  pre: ({ children }) => <CodeBlock {...fenceFrom(children)} />,
  table: (props) => (
    <div className="my-6 overflow-x-auto rounded-lg border border-line">
      <table className="w-full border-collapse text-left text-sm" {...props} />
    </div>
  ),
  th: (props) => (
    <th
      className="border-line border-b bg-faint px-4 py-2.5 font-medium text-xs uppercase tracking-wider"
      {...props}
    />
  ),
  td: (props) => (
    <td
      className="border-line/60 border-b px-4 py-3 align-top text-muted"
      {...props}
    />
  ),
  blockquote: (props) => (
    <blockquote
      className="my-6 border-accent/40 border-l-2 pl-4 text-muted italic"
      {...props}
    />
  ),
  hr: () => <hr className="my-10 border-line" />,
};

export function useMDXComponents(): MDXComponents {
  return components;
}
