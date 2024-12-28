/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config) => {
    config.experiments = {
      asyncWebAssembly: true,
      layers: true,
    };
    // Explicitly handle WASM files
    config.module.rules.push({
      test: /\.wasm$/,
      type: "webassembly/async",
    });
    // Handle loading of WASM JavaScript glue code
    config.module.rules.push({
      test: /volume_viewer\.js$/,
      type: "javascript/auto",
    });
    return config;
  },
};

module.exports = nextConfig;
