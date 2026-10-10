// hardhat-exposed generates its wrappers from the AST of the default profile, so solc stays there.
import hardhatSlangSolx from '@nomicfoundation/hardhat-slang-solx';

import baseConfig from './hardhat.config.base.ts';

const solidity = baseConfig.solidity as { version: string; settings: { evmVersion: string } };

export default {
  ...baseConfig,
  plugins: [...baseConfig.plugins, hardhatSlangSolx],
  solidity: {
    profiles: {
      default: solidity,
      ...(process.env.USE_SOLX === 'true'
        ? {
            'slang-solx': {
              type: 'slang-solx',
              version: solidity.version,
              path: process.env.SOLX,
              settings: {
                evmVersion: solidity.settings.evmVersion,
                viaIR: process.env.VIA_IR === 'true',
              },
            },
          }
        : {}),
    },
  },
  test: {
    ...baseConfig.test,
    mocha: {
      reporter: 'json',
      reporterOptions: { output: process.env.JUNIT_REPORT },
    },
  },
};
