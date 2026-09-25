import type { NextConfig } from "next";

const basePath = "/VN_Rust";

if (process.env.NODE_ENV === "development") {
  console.log(
    `- Docs:          http://localhost:${process.env.PORT ?? 3000}${basePath}`,
  );
}

const nextConfig: NextConfig = {
  output: "export",
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
