import type { Config } from "@jest/types";

const config: Config.InitialOptions = {
  verbose: true,
  transform: {
    "^.+\\.(t|j)sx?$": [
      "@swc/jest",
      {
        jsc: {
          transform: {
            react: {
              runtime: "automatic",
            },
          },
        },
      },
    ],
  },
  setupFilesAfterEnv: ["<rootDir>/test/jest-setup.ts"],
  testEnvironment: "jest-environment-jsdom",
  collectCoverageFrom: ["<rootDir>/src/**/*"],
  globals: {
    TESTING: true,
  },
};
export default config;
