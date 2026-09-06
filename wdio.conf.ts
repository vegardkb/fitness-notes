/// <reference types="@wdio/tauri-service" />
// Build the test binary first (separate identifier + wdio fixtures):
//   pnpm build:wdio
// then run:
//   pnpm wdio
export const config: WebdriverIO.Config = {
    runner: 'local',
    tsConfigPath: './test/tsconfig.json',
    specs: ['./test/specs/**/*.ts'],
    // one app instance — the service launches the real binary with an
    // embedded WebDriver server, so parallel instances would clash on ports
    maxInstances: 1,

    capabilities: [{
        browserName: 'tauri'
    }],
    logLevel: 'info',
    bail: 0,
    waitforTimeout: 10000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    services: [[
        '@wdio/tauri-service',
        {
            appBinaryPath: './src-tauri/target/release/fitness-notes'
        }
    ]],
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 30000
    },
}
