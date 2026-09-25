import createMDX from "@next/mdx";
import type { NextConfig } from "next";

const basePath = process.env.NODE_ENV === "production" ? "/novn" : "";

const nextConfig: NextConfig = {
  output: "export",
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
  pageExtensions: ["ts", "tsx", "mdx"],
  serverExternalPackages: ["web-tree-sitter"],
  env: { NEXT_PUBLIC_BASE_PATH: basePath },
};

export default createMDX({
  options: {
    remarkPlugins: ["remark-gfm"],
    rehypePlugins: ["rehype-slug"],
  },
})(nextConfig);
