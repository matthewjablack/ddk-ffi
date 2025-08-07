const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');
const path = require('path');

const root = path.resolve(__dirname, '..');

const config = {
  watchFolders: [root],
  resolver: {
    alias: {
      '@bennyhodl/ddk-rn': path.resolve(root, 'src'),
    },
    unstable_enablePackageExports: true,
  },
};

module.exports = mergeConfig(getDefaultConfig(__dirname), config);
