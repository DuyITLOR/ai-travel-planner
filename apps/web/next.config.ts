import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // packages/shared là TypeScript thô, để Next tự biên dịch.
  transpilePackages: ['@atp/shared'],
};

export default nextConfig;
