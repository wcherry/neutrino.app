import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  transpilePackages: ['@neutrino/ui'],
  // async rewrites() {
  //   return [
  //     {
  //       source: '/api/:path*',
  //       destination: 'http://0.0.0.0:8880/api/:path*',
  //     },
  //   ];
  // },
  experimental: {
    // Optimize CSS imports from workspace packages
  },
  output: "export",
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: '**',
      },
    ],
  },
};

export default nextConfig;
