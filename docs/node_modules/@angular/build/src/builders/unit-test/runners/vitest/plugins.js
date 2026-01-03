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
exports.createVitestConfigPlugin = createVitestConfigPlugin;
exports.createVitestPlugins = createVitestPlugins;
const node_assert_1 = __importDefault(require("node:assert"));
const promises_1 = require("node:fs/promises");
const node_module_1 = require("node:module");
const node_os_1 = require("node:os");
const node_path_1 = __importDefault(require("node:path"));
const assets_middleware_1 = require("../../../../tools/vite/middlewares/assets-middleware");
const path_1 = require("../../../../utils/path");
async function findTestEnvironment(projectResolver) {
    try {
        projectResolver('happy-dom');
        return 'happy-dom';
    }
    catch {
        // happy-dom is not installed, fallback to jsdom
        return 'jsdom';
    }
}
async function createVitestConfigPlugin(options) {
    const { include, browser, projectName, reporters, setupFiles, projectPlugins, projectSourceRoot, } = options;
    const { mergeConfig } = await Promise.resolve().then(() => __importStar(require('vitest/config')));
    return {
        name: 'angular:vitest-configuration',
        async config(config) {
            const testConfig = config.test;
            if (testConfig?.projects?.length) {
                this.warn('The "test.projects" option in the Vitest configuration file is not supported. ' +
                    'The Angular CLI Test system will construct its own project configuration.');
                delete testConfig.projects;
            }
            if (testConfig?.include) {
                this.warn('The "test.include" option in the Vitest configuration file is not supported. ' +
                    'The Angular CLI Test system will manage test file discovery.');
                delete testConfig.include;
            }
            // Merge user-defined plugins from the Vitest config with the CLI's internal plugins.
            if (config.plugins) {
                const userPlugins = config.plugins.filter((plugin) => 
                // Only inspect objects with a `name` property as these would be the internal injected plugins
                !plugin ||
                    typeof plugin !== 'object' ||
                    !('name' in plugin) ||
                    (!plugin.name.startsWith('angular:') && !plugin.name.startsWith('vitest')));
                if (userPlugins.length > 0) {
                    projectPlugins.push(...userPlugins);
                }
                delete config.plugins;
            }
            const projectResolver = (0, node_module_1.createRequire)(projectSourceRoot + '/').resolve;
            const projectDefaults = {
                test: {
                    setupFiles,
                    globals: true,
                    // Default to `false` to align with the Karma/Jasmine experience.
                    isolate: false,
                    sequence: { setupFiles: 'list' },
                },
                optimizeDeps: {
                    noDiscovery: true,
                    include: options.optimizeDepsInclude,
                },
                resolve: {
                    mainFields: ['es2020', 'module', 'main'],
                    conditions: ['es2015', 'es2020', 'module', ...(browser ? ['browser'] : [])],
                },
            };
            const { optimizeDeps, resolve } = config;
            const projectOverrides = {
                test: {
                    name: projectName,
                    include,
                    // CLI provider browser options override, if present
                    ...(browser ? { browser } : {}),
                    // If the user has not specified an environment, use a smart default.
                    ...(!testConfig?.environment
                        ? { environment: await findTestEnvironment(projectResolver) }
                        : {}),
                },
                plugins: projectPlugins,
                optimizeDeps,
                resolve,
            };
            const projectBase = mergeConfig(projectDefaults, testConfig ? { test: testConfig } : {});
            const projectConfig = mergeConfig(projectBase, projectOverrides);
            return {
                test: {
                    coverage: await generateCoverageOption(options.coverage, testConfig?.coverage, projectName),
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    ...(reporters ? { reporters } : {}),
                    projects: [projectConfig],
                },
            };
        },
    };
}
async function loadResultFile(file) {
    if (file.origin === 'memory') {
        return new TextDecoder('utf-8').decode(file.contents);
    }
    return (0, promises_1.readFile)(file.inputPath, 'utf-8');
}
function createVitestPlugins(pluginOptions) {
    const { workspaceRoot, buildResultFiles, testFileToEntryPoint } = pluginOptions;
    const isWindows = (0, node_os_1.platform)() === 'win32';
    return [
        {
            name: 'angular:test-in-memory-provider',
            enforce: 'pre',
            resolveId: (id, importer) => {
                // Fast path for test entry points.
                if (testFileToEntryPoint.has(id)) {
                    return id;
                }
                // Workaround for Vitest in Windows when a fully qualified absolute path is provided with
                // a superfluous leading slash. This can currently occur with the `@vitest/coverage-v8` provider
                // when it uses `removeStartsWith(url, FILE_PROTOCOL)` to convert a file URL resulting in
                // `/D:/tmp_dir/...` instead of `D:/tmp_dir/...`.
                if (id[0] === '/' && isWindows) {
                    const slicedId = id.slice(1);
                    if (node_path_1.default.isAbsolute(slicedId)) {
                        return slicedId;
                    }
                }
                if (importer && (id[0] === '.' || id[0] === '/')) {
                    let fullPath;
                    if (testFileToEntryPoint.has(importer)) {
                        fullPath = (0, path_1.toPosixPath)(node_path_1.default.join(workspaceRoot, id));
                    }
                    else {
                        fullPath = (0, path_1.toPosixPath)(node_path_1.default.join(node_path_1.default.dirname(importer), id));
                    }
                    const relativePath = node_path_1.default.relative(workspaceRoot, fullPath);
                    if (buildResultFiles.has((0, path_1.toPosixPath)(relativePath))) {
                        return fullPath;
                    }
                }
                // Determine the base directory for resolution.
                let baseDir;
                if (importer) {
                    // If the importer is a test entry point, resolve relative to the workspace root.
                    // Otherwise, resolve relative to the importer's directory.
                    baseDir = testFileToEntryPoint.has(importer) ? workspaceRoot : node_path_1.default.dirname(importer);
                }
                else {
                    // If there's no importer, assume the id is relative to the workspace root.
                    baseDir = workspaceRoot;
                }
                // Construct the full, absolute path and normalize it to POSIX format.
                const fullPath = (0, path_1.toPosixPath)(node_path_1.default.join(baseDir, id));
                // Check if the resolved path corresponds to a known build artifact.
                const relativePath = node_path_1.default.relative(workspaceRoot, fullPath);
                if (buildResultFiles.has((0, path_1.toPosixPath)(relativePath))) {
                    return fullPath;
                }
                // If the module cannot be resolved from the build artifacts, let other plugins handle it.
                return undefined;
            },
            load: async (id) => {
                (0, node_assert_1.default)(buildResultFiles.size > 0, 'buildResult must be available for in-memory loading.');
                // Attempt to load as a source test file.
                const entryPoint = testFileToEntryPoint.get(id);
                let outputPath;
                if (entryPoint) {
                    outputPath = entryPoint + '.js';
                    // To support coverage exclusion of the actual test file, the virtual
                    // test entry point only references the built and bundled intermediate file.
                    return {
                        code: `import "./${outputPath}";`,
                    };
                }
                else {
                    // Attempt to load as a built artifact.
                    const relativePath = node_path_1.default.relative(workspaceRoot, id);
                    outputPath = (0, path_1.toPosixPath)(relativePath);
                }
                const outputFile = buildResultFiles.get(outputPath);
                if (outputFile) {
                    const code = await loadResultFile(outputFile);
                    const sourceMapPath = outputPath + '.map';
                    const sourceMapFile = buildResultFiles.get(sourceMapPath);
                    const sourceMapText = sourceMapFile ? await loadResultFile(sourceMapFile) : undefined;
                    // Vitest will include files in the coverage report if the sourcemap contains no sources.
                    // For builder-internal generated code chunks, which are typically helper functions,
                    // a virtual source is added to the sourcemap to prevent them from being incorrectly
                    // included in the final coverage report.
                    const map = sourceMapText ? JSON.parse(sourceMapText) : undefined;
                    if (map) {
                        if (!map.sources?.length && !map.sourcesContent?.length && !map.mappings) {
                            map.sources = ['virtual:builder'];
                        }
                    }
                    return {
                        code,
                        map,
                    };
                }
            },
            configureServer: (server) => {
                server.middlewares.use((0, assets_middleware_1.createBuildAssetsMiddleware)(server.config.base, buildResultFiles));
            },
        },
        {
            name: 'angular:html-index',
            transformIndexHtml: () => {
                // Add all global stylesheets
                if (buildResultFiles.has('styles.css')) {
                    return [
                        {
                            tag: 'link',
                            attrs: { href: 'styles.css', rel: 'stylesheet' },
                            injectTo: 'head',
                        },
                    ];
                }
                return [];
            },
        },
    ];
}
async function generateCoverageOption(optionsCoverage, configCoverage, projectName) {
    let defaultExcludes = [];
    if (optionsCoverage.exclude) {
        try {
            const vitestConfig = await Promise.resolve().then(() => __importStar(require('vitest/config')));
            defaultExcludes = vitestConfig.coverageConfigDefaults.exclude;
        }
        catch { }
    }
    return {
        excludeAfterRemap: true,
        reportsDirectory: configCoverage?.reportsDirectory ?? (0, path_1.toPosixPath)(node_path_1.default.join('coverage', projectName)),
        ...(optionsCoverage.enabled !== undefined ? { enabled: optionsCoverage.enabled } : {}),
        // Vitest performs a pre-check and a post-check for sourcemaps.
        // The pre-check uses the bundled files, so specific bundled entry points and chunks need to be included.
        // The post-check uses the original source files, so the user's include is used.
        ...(optionsCoverage.include
            ? { include: ['spec-*.js', 'chunk-*.js', ...optionsCoverage.include] }
            : {}),
        thresholds: optionsCoverage.thresholds,
        watermarks: optionsCoverage.watermarks,
        // Special handling for `exclude`/`reporters` due to an undefined value causing upstream failures
        ...(optionsCoverage.exclude
            ? {
                exclude: [
                    // Augment the default exclude https://vitest.dev/config/#coverage-exclude
                    // with the user defined exclusions
                    ...optionsCoverage.exclude,
                    ...defaultExcludes,
                ],
            }
            : {}),
        ...(optionsCoverage.reporters
            ? { reporter: optionsCoverage.reporters }
            : {}),
    };
}
//# sourceMappingURL=plugins.js.map