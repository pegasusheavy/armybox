"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.VitestExecutor = void 0;
const node_assert_1 = __importDefault(require("node:assert"));
const node_path_1 = __importDefault(require("node:path"));
const utils_1 = require("../../../../tools/vite/utils");
const error_1 = require("../../../../utils/error");
const results_1 = require("../../../application/results");
const browser_provider_1 = require("./browser-provider");
const configuration_1 = require("./configuration");
const plugins_1 = require("./plugins");
class VitestExecutor {
    vitest;
    normalizePath;
    projectName;
    options;
    buildResultFiles = new Map();
    externalMetadata = {
        implicitBrowser: [],
        implicitServer: [],
        explicitBrowser: [],
        explicitServer: [],
    };
    // This is a reverse map of the entry points created in `build-options.ts`.
    // It is used by the in-memory provider plugin to map the requested test file
    // path back to its bundled output path.
    // Example: `Map<'/path/to/src/app.spec.ts', 'spec-src-app-spec'>`
    testFileToEntryPoint = new Map();
    entryPointToTestFile = new Map();
    constructor(projectName, options, testEntryPointMappings) {
        this.projectName = projectName;
        this.options = options;
        if (testEntryPointMappings) {
            for (const [entryPoint, testFile] of testEntryPointMappings) {
                this.testFileToEntryPoint.set(testFile, entryPoint);
                this.entryPointToTestFile.set(entryPoint + '.js', testFile);
            }
        }
    }
    async *execute(buildResult) {
        this.normalizePath ??= (await Promise.resolve().then(() => __importStar(require('vite')))).normalizePath;
        if (buildResult.kind === results_1.ResultKind.Full) {
            this.buildResultFiles.clear();
            for (const [path, file] of Object.entries(buildResult.files)) {
                this.buildResultFiles.set(this.normalizePath(path), file);
            }
        }
        else {
            for (const file of buildResult.removed) {
                this.buildResultFiles.delete(this.normalizePath(file.path));
            }
            for (const [path, file] of Object.entries(buildResult.files)) {
                this.buildResultFiles.set(this.normalizePath(path), file);
            }
        }
        (0, utils_1.updateExternalMetadata)(buildResult, this.externalMetadata, undefined, true);
        // Initialize Vitest if not already present.
        this.vitest ??= await this.initializeVitest();
        const vitest = this.vitest;
        let testResults;
        if (buildResult.kind === results_1.ResultKind.Incremental) {
            // To rerun tests, Vitest needs the original test file paths, not the output paths.
            const modifiedSourceFiles = new Set();
            for (const modifiedFile of [...buildResult.modified, ...buildResult.added]) {
                // The `modified` files in the build result are the output paths.
                // We need to find the original source file path to pass to Vitest.
                const source = this.entryPointToTestFile.get(modifiedFile);
                if (source) {
                    modifiedSourceFiles.add(source);
                }
                vitest.invalidateFile(this.normalizePath(node_path_1.default.join(this.options.workspaceRoot, modifiedFile)));
            }
            const specsToRerun = [];
            for (const file of modifiedSourceFiles) {
                vitest.invalidateFile(file);
                const specs = vitest.getModuleSpecifications(file);
                if (specs) {
                    specsToRerun.push(...specs);
                }
            }
            if (specsToRerun.length > 0) {
                testResults = await vitest.rerunTestSpecifications(specsToRerun);
            }
        }
        // Check if all the tests pass to calculate the result
        const testModules = testResults?.testModules ?? this.vitest.state.getTestModules();
        yield { success: testModules.every((testModule) => testModule.ok()) };
    }
    async [Symbol.asyncDispose]() {
        await this.vitest?.close();
    }
    prepareSetupFiles() {
        const { setupFiles } = this.options;
        // Add setup file entries for TestBed initialization and project polyfills
        const testSetupFiles = ['init-testbed.js', ...setupFiles];
        // TODO: Provide additional result metadata to avoid needing to extract based on filename
        if (this.buildResultFiles.has('polyfills.js')) {
            testSetupFiles.unshift('polyfills.js');
        }
        return testSetupFiles;
    }
    async initializeVitest() {
        const { coverage, reporters, outputFile, workspaceRoot, browsers, debug, watch, browserViewport, ui, projectRoot, runnerConfig, projectSourceRoot, cacheOptions, } = this.options;
        const projectName = this.projectName;
        let vitestNodeModule;
        try {
            vitestNodeModule = await Promise.resolve().then(() => __importStar(require('vitest/node')));
        }
        catch (error) {
            (0, error_1.assertIsError)(error);
            if (error.code !== 'ERR_MODULE_NOT_FOUND') {
                throw error;
            }
            throw new Error('The `vitest` package was not found. Please install the package and rerun the test command.');
        }
        const { startVitest } = vitestNodeModule;
        // Setup vitest browser options if configured
        const browserOptions = await (0, browser_provider_1.setupBrowserConfiguration)(browsers, debug, projectSourceRoot, browserViewport);
        if (browserOptions.errors?.length) {
            throw new Error(browserOptions.errors.join('\n'));
        }
        (0, node_assert_1.default)(this.buildResultFiles.size > 0, 'buildResult must be available before initializing vitest');
        const testSetupFiles = this.prepareSetupFiles();
        const projectPlugins = (0, plugins_1.createVitestPlugins)({
            workspaceRoot,
            projectSourceRoot,
            projectName,
            buildResultFiles: this.buildResultFiles,
            testFileToEntryPoint: this.testFileToEntryPoint,
        });
        const debugOptions = debug
            ? {
                inspectBrk: true,
                isolate: false,
                fileParallelism: false,
            }
            : {};
        const externalConfigPath = runnerConfig === true
            ? await (0, configuration_1.findVitestBaseConfig)([projectRoot, workspaceRoot])
            : runnerConfig;
        return startVitest('test', undefined, {
            config: externalConfigPath,
            root: workspaceRoot,
            project: projectName,
            outputFile,
            cache: cacheOptions.enabled ? undefined : false,
            testNamePattern: this.options.filter,
            watch,
            ...(typeof ui === 'boolean' ? { ui } : {}),
            ...debugOptions,
        }, {
            // Note `.vitest` is auto appended to the path.
            cacheDir: cacheOptions.path,
            server: {
                // Disable the actual file watcher. The boolean watch option above should still
                // be enabled as it controls other internal behavior related to rerunning tests.
                watch: null,
            },
            plugins: [
                await (0, plugins_1.createVitestConfigPlugin)({
                    browser: browserOptions.browser,
                    coverage,
                    projectName,
                    projectSourceRoot,
                    optimizeDepsInclude: this.externalMetadata.implicitBrowser,
                    reporters,
                    setupFiles: testSetupFiles,
                    projectPlugins,
                    include: [...this.testFileToEntryPoint.keys()].filter(
                    // Filter internal entries
                    (entry) => !entry.startsWith('angular:')),
                }),
            ],
        });
    }
}
exports.VitestExecutor = VitestExecutor;
//# sourceMappingURL=executor.js.map