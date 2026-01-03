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
Object.defineProperty(exports, "__esModule", { value: true });
exports.setupBrowserConfiguration = setupBrowserConfiguration;
const node_module_1 = require("node:module");
const error_1 = require("../../../../utils/error");
function findBrowserProvider(projectResolver) {
    const requiresPreview = !!process.versions.webcontainer;
    // One of these must be installed in the project to use browser testing
    const vitestBuiltinProviders = requiresPreview
        ? ['preview']
        : ['playwright', 'webdriverio', 'preview'];
    for (const providerName of vitestBuiltinProviders) {
        try {
            projectResolver(`@vitest/browser-${providerName}`);
            return providerName;
        }
        catch { }
    }
    return undefined;
}
function normalizeBrowserName(browserName) {
    // Normalize browser names to match Vitest's expectations for headless but also supports karma's names
    // e.g., 'ChromeHeadless' -> 'chrome', 'FirefoxHeadless' -> 'firefox'
    // and 'Chrome' -> 'chrome', 'Firefox' -> 'firefox'.
    const normalized = browserName.toLowerCase();
    const headless = normalized.endsWith('headless');
    return {
        browser: headless ? normalized.slice(0, -8) : normalized,
        headless: headless,
    };
}
async function setupBrowserConfiguration(browsers, debug, projectSourceRoot, viewport) {
    if (browsers === undefined) {
        return {};
    }
    const projectResolver = (0, node_module_1.createRequire)(projectSourceRoot + '/').resolve;
    let errors;
    const providerName = findBrowserProvider(projectResolver);
    if (!providerName) {
        errors ??= [];
        errors.push('The "browsers" option requires either "playwright" or "webdriverio" to be installed within the project.' +
            ' Please install one of these packages and rerun the test command.');
    }
    let provider;
    if (providerName) {
        const providerPackage = `@vitest/browser-${providerName}`;
        try {
            const providerModule = await Promise.resolve(`${projectResolver(providerPackage)}`).then(s => __importStar(require(s)));
            // Validate that the imported module has the expected structure
            const providerFactory = providerModule[providerName];
            if (typeof providerFactory === 'function') {
                if (providerName === 'playwright' &&
                    process.env['CHROME_BIN']?.includes('rules_browsers')) {
                    // Use the Chrome binary from the 'rules_browsers' toolchain (via CHROME_BIN)
                    // for Playwright when available to ensure hermetic testing, preventing reliance
                    // on locally installed or NPM-managed browser versions.
                    provider = providerFactory({
                        launchOptions: {
                            executablePath: process.env.CHROME_BIN,
                        },
                    });
                }
                else {
                    provider = providerFactory();
                }
            }
            else {
                errors ??= [];
                errors.push(`The "${providerPackage}" package does not have a valid browser provider export.`);
            }
        }
        catch (e) {
            (0, error_1.assertIsError)(e);
            errors ??= [];
            // Check for a module not found error to provide a more specific message
            if (e.code === 'ERR_MODULE_NOT_FOUND') {
                errors.push(`The "browsers" option with "${providerName}" requires the "${providerPackage}" package.` +
                    ' Please install this package and rerun the test command.');
            }
            else {
                // Handle other potential errors during import
                errors.push(`An error occurred while loading the "${providerPackage}" browser provider:\n  ${e.message}`);
            }
        }
    }
    if (errors) {
        return { errors };
    }
    const isCI = !!process.env['CI'];
    const instances = browsers.map(normalizeBrowserName);
    if (providerName === 'preview') {
        instances.forEach((instance) => {
            instance.headless = false;
        });
    }
    else if (isCI) {
        instances.forEach((instance) => {
            instance.headless = true;
        });
    }
    const browser = {
        enabled: true,
        provider,
        ui: !isCI && instances.some((instance) => !instance.headless),
        viewport,
        instances,
    };
    return { browser };
}
//# sourceMappingURL=browser-provider.js.map