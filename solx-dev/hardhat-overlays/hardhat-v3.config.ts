import hardhatSlangSolx from '@nomicfoundation/hardhat-slang-solx';

import baseConfig from './hardhat.config.base.ts';

const { path: _path, ...defaultProfile } = baseConfig.solidity.profiles.default as {
  version: string;
  path?: string;
};

export default {
  ...baseConfig,
  plugins: [...baseConfig.plugins, hardhatSlangSolx],
  solidity: {
    profiles: {
      default: defaultProfile,
      ...(process.env.USE_SOLX === 'true'
        ? {
            'slang-solx': {
              type: 'slang-solx',
              version: defaultProfile.version,
              path: process.env.SOLX,
              settings: { viaIR: process.env.VIA_IR === 'true' },
            },
          }
        : {}),
    },
  },
};
