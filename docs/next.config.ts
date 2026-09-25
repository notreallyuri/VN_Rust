import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  basePath: process.env.NODE_ENV === "production" ? "/VN_Rust" : "",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
