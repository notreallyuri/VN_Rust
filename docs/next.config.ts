import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  basePath: "/VN_Rust",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
