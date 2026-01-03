"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.PACKAGE_MANAGER_PRECEDENCE = exports.SUPPORTED_PACKAGE_MANAGERS = void 0;
const parsers_1 = require("./parsers");
/**
 * A map of supported package managers to their descriptors.
 * This is the single source of truth for all package-manager-specific
 * configuration and behavior.
 *
 * Each descriptor is intentionally explicit and self-contained. This approach
 * avoids inheritance or fallback logic between package managers, ensuring that
 * the behavior for each one is clear, predictable, and easy to modify in
 * isolation. For example, `yarn-classic` does not inherit any properties from
 * the `yarn` descriptor; it is a complete and independent definition.
 */
exports.SUPPORTED_PACKAGE_MANAGERS = {
    npm: {
        binary: 'npm',
        lockfiles: ['package-lock.json', 'npm-shrinkwrap.json'],
        addCommand: 'install',
        installCommand: ['install'],
        forceFlag: '--force',
        saveExactFlag: '--save-exact',
        saveTildeFlag: '--save-tilde',
        saveDevFlag: '--save-dev',
        noLockfileFlag: '--no-package-lock',
        ignoreScriptsFlag: '--ignore-scripts',
        getRegistryOptions: (registry) => ({ args: ['--registry', registry] }),
        versionCommand: ['--version'],
        listDependenciesCommand: ['list', '--depth=0', '--json=true', '--all=true'],
        getManifestCommand: ['view', '--json'],
        viewCommandFieldArgFormatter: (fields) => [...fields],
        outputParsers: {
            listDependencies: parsers_1.parseNpmLikeDependencies,
            getPackageManifest: parsers_1.parseNpmLikeManifest,
            getRegistryMetadata: parsers_1.parseNpmLikeMetadata,
        },
    },
    yarn: {
        binary: 'yarn',
        lockfiles: ['yarn.lock'],
        addCommand: 'add',
        installCommand: ['install'],
        forceFlag: '--force',
        saveExactFlag: '--exact',
        saveTildeFlag: '--tilde',
        saveDevFlag: '--dev',
        noLockfileFlag: '--no-lockfile',
        ignoreScriptsFlag: '--ignore-scripts',
        getRegistryOptions: (registry) => ({ env: { NPM_CONFIG_REGISTRY: registry } }),
        versionCommand: ['--version'],
        listDependenciesCommand: ['list', '--depth=0', '--json', '--recursive=false'],
        getManifestCommand: ['npm', 'info', '--json'],
        viewCommandFieldArgFormatter: (fields) => ['--fields', fields.join(',')],
        outputParsers: {
            listDependencies: parsers_1.parseYarnModernDependencies,
            getPackageManifest: parsers_1.parseNpmLikeManifest,
            getRegistryMetadata: parsers_1.parseNpmLikeMetadata,
        },
    },
    'yarn-classic': {
        binary: 'yarn',
        // This is intentionally empty. `yarn-classic` is not a discoverable package manager.
        // The discovery process finds `yarn` via `yarn.lock`, and the factory logic
        // determines whether it is classic or modern by checking the installed version.
        lockfiles: [],
        addCommand: 'add',
        installCommand: ['install'],
        forceFlag: '--force',
        saveExactFlag: '--exact',
        saveTildeFlag: '--tilde',
        saveDevFlag: '--dev',
        noLockfileFlag: '--no-lockfile',
        ignoreScriptsFlag: '--ignore-scripts',
        getRegistryOptions: (registry) => ({ args: ['--registry', registry] }),
        versionCommand: ['--version'],
        listDependenciesCommand: ['list', '--depth=0', '--json'],
        getManifestCommand: ['info', '--json'],
        outputParsers: {
            listDependencies: parsers_1.parseYarnClassicDependencies,
            getPackageManifest: parsers_1.parseYarnLegacyManifest,
            getRegistryMetadata: parsers_1.parseNpmLikeMetadata,
        },
    },
    pnpm: {
        binary: 'pnpm',
        lockfiles: ['pnpm-lock.yaml'],
        addCommand: 'add',
        installCommand: ['install'],
        forceFlag: '--force',
        saveExactFlag: '--save-exact',
        saveTildeFlag: '--save-tilde',
        saveDevFlag: '--save-dev',
        noLockfileFlag: '--no-lockfile',
        ignoreScriptsFlag: '--ignore-scripts',
        getRegistryOptions: (registry) => ({ args: ['--registry', registry] }),
        versionCommand: ['--version'],
        listDependenciesCommand: ['list', '--depth=0', '--json'],
        getManifestCommand: ['view', '--json'],
        viewCommandFieldArgFormatter: (fields) => [...fields],
        outputParsers: {
            listDependencies: parsers_1.parseNpmLikeDependencies,
            getPackageManifest: parsers_1.parseNpmLikeManifest,
            getRegistryMetadata: parsers_1.parseNpmLikeMetadata,
        },
    },
    bun: {
        binary: 'bun',
        lockfiles: ['bun.lockb', 'bun.lock'],
        addCommand: 'add',
        installCommand: ['install'],
        forceFlag: '--force',
        saveExactFlag: '--exact',
        saveTildeFlag: '', // Bun does not have a flag for tilde, it defaults to caret.
        saveDevFlag: '--development',
        noLockfileFlag: '', // Bun does not have a flag for this.
        ignoreScriptsFlag: '--ignore-scripts',
        getRegistryOptions: (registry) => ({ args: ['--registry', registry] }),
        versionCommand: ['--version'],
        listDependenciesCommand: ['pm', 'ls', '--json'],
        getManifestCommand: ['pm', 'view', '--json'],
        viewCommandFieldArgFormatter: (fields) => [...fields],
        outputParsers: {
            listDependencies: parsers_1.parseNpmLikeDependencies,
            getPackageManifest: parsers_1.parseNpmLikeManifest,
            getRegistryMetadata: parsers_1.parseNpmLikeMetadata,
        },
    },
};
/**
 * The order of precedence for package managers.
 * This is a best-effort ordering based on estimated Angular community usage and default presence.
 */
exports.PACKAGE_MANAGER_PRECEDENCE = [
    'pnpm',
    'yarn',
    'bun',
    'npm',
];
//# sourceMappingURL=package-manager-descriptor.js.map