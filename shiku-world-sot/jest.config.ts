import type { Config } from 'jest';

const config: Config = {
  moduleFileExtensions: ['js', 'json', 'ts'],
  transform: {
    '^.+\\.ts$': 'ts-jest',
  },
  testMatch: ['**/*.spec.ts'],
  testEnvironment: 'jsdom',
};

export default config;
