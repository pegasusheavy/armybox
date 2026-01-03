"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.PackageManager = void 0;
/**
 * @fileoverview This file contains the `PackageManager` class, which is the
 * core execution engine for all package manager commands. It is designed to be
 * a flexible and secure abstraction over the various package managers.
 */
const node_path_1 = require("node:path");
const error_1 = require("./error");
/**
 * The fields to request from the registry for package metadata.
 * This is a performance optimization to avoid downloading the full manifest
 * when only summary data (like versions and tags) is needed.
 */
const METADATA_FIELDS = ['name', 'dist-tags', 'versions', 'time'];
/**
 * The fields to request from the registry for a package's manifest.
 * This is a performance optimization to avoid downloading unnecessary data.
 * These fields are the ones required by the CLI for operations like `ng add` and `ng update`.
 */
const MANIFEST_FIELDS = [
    'name',
    'version',
    'deprecated',
    'dependencies',
    'peerDependencies',
    'devDependencies',
    'homepage',
    'schematics',
    'ng-add',
    'ng-update',
];
/**
 * A class that provides a high-level, package-manager-agnostic API for
 * interacting with a project's dependencies.
 *
 * This class is an implementation of the Strategy design pattern. It is
 * instantiated with a `PackageManagerDescriptor` that defines the specific
 * commands and flags for a given package manager.
 */
class PackageManager {
    host;
    cwd;
    descriptor;
    options;
    #manifestCache = new Map();
    #metadataCache = new Map();
    #dependencyCache = null;
    /**
     * Creates a new `PackageManager` instance.
     * @param host A `Host` instance for interacting with the file system and running commands.
     * @param cwd The absolute path to the project's working directory.
     * @param descriptor A `PackageManagerDescriptor` that defines the commands for a specific package manager.
     * @param options An options object to configure the instance.
     */
    constructor(host, cwd, descriptor, options = {}) {
        this.host = host;
        this.cwd = cwd;
        this.descriptor = descriptor;
        this.options = options;
        if (this.options.dryRun && !this.options.logger) {
            throw new Error('A logger must be provided when dryRun is enabled.');
        }
    }
    /**
     * The name of the package manager's binary.
     */
    get name() {
        return this.descriptor.binary;
    }
    /**
     * A private method to lazily populate the dependency cache.
     * This is a performance optimization to avoid running `npm list` multiple times.
     * @returns A promise that resolves to the dependency cache map.
     */
    async #populateDependencyCache() {
        if (this.#dependencyCache !== null) {
            return this.#dependencyCache;
        }
        const args = this.descriptor.listDependenciesCommand;
        const dependencies = await this.#fetchAndParse(args, (stdout, logger) => this.descriptor.outputParsers.listDependencies(stdout, logger));
        return (this.#dependencyCache = dependencies ?? new Map());
    }
    /**
     * A private method to run a command using the package manager's binary.
     * @param args The arguments to pass to the command.
     * @param options Options for the child process.
     * @returns A promise that resolves with the standard output and standard error of the command.
     */
    async #run(args, options = {}) {
        const { registry, cwd, ...runOptions } = options;
        const finalArgs = [...args];
        let finalEnv;
        if (registry) {
            const registryOptions = this.descriptor.getRegistryOptions?.(registry);
            if (!registryOptions) {
                throw new Error(`The configured package manager, '${this.descriptor.binary}', does not support a custom registry.`);
            }
            if (registryOptions.args) {
                finalArgs.push(...registryOptions.args);
            }
            if (registryOptions.env) {
                finalEnv = registryOptions.env;
            }
        }
        const executionDirectory = cwd ?? this.cwd;
        if (this.options.dryRun) {
            this.options.logger?.info(`[DRY RUN] Would execute in [${executionDirectory}]: ${this.descriptor.binary} ${finalArgs.join(' ')}`);
            return { stdout: '', stderr: '' };
        }
        return this.host.runCommand(this.descriptor.binary, finalArgs, {
            ...runOptions,
            cwd: executionDirectory,
            stdio: 'pipe',
            env: finalEnv,
        });
    }
    /**
     * A private, generic method to encapsulate the common logic of running a command,
     * handling errors, and parsing the output.
     * @param args The arguments to pass to the command.
     * @param parser A function that parses the command's stdout.
     * @param options Options for the command, including caching.
     * @returns A promise that resolves to the parsed data, or null if not found.
     */
    async #fetchAndParse(args, parser, options = {}) {
        const { cache, cacheKey, bypassCache, ...runOptions } = options;
        if (!bypassCache && cache && cacheKey && cache.has(cacheKey)) {
            return cache.get(cacheKey);
        }
        let stdout;
        let stderr;
        try {
            ({ stdout, stderr } = await this.#run(args, runOptions));
        }
        catch (e) {
            if (e instanceof error_1.PackageManagerError && typeof e.exitCode === 'number' && e.exitCode !== 0) {
                // Some package managers exit with a non-zero code when the package is not found.
                if (cache && cacheKey) {
                    cache.set(cacheKey, null);
                }
                return null;
            }
            throw e;
        }
        try {
            const result = parser(stdout, this.options.logger);
            if (cache && cacheKey) {
                cache.set(cacheKey, result);
            }
            return result;
        }
        catch (e) {
            const message = `Failed to parse package manager output: ${e instanceof Error ? e.message : ''}`;
            throw new error_1.PackageManagerError(message, stdout, stderr, 0);
        }
    }
    /**
     * Adds a package to the project's dependencies.
     * @param packageName The name of the package to add.
     * @param save The save strategy to use.
     * - `exact`: The package will be saved with an exact version.
     * - `tilde`: The package will be saved with a tilde version range (`~`).
     * - `none`: The package will be saved with the default version range (`^`).
     * @param asDevDependency Whether to install the package as a dev dependency.
     * @param noLockfile Whether to skip updating the lockfile.
     * @param options Extra options for the command.
     * @returns A promise that resolves when the command is complete.
     */
    async add(packageName, save, asDevDependency, noLockfile, ignoreScripts, options = {}) {
        const flags = [
            asDevDependency ? this.descriptor.saveDevFlag : '',
            save === 'exact' ? this.descriptor.saveExactFlag : '',
            save === 'tilde' ? this.descriptor.saveTildeFlag : '',
            noLockfile ? this.descriptor.noLockfileFlag : '',
            ignoreScripts ? this.descriptor.ignoreScriptsFlag : '',
        ].filter((flag) => flag);
        const args = [this.descriptor.addCommand, packageName, ...flags];
        await this.#run(args, options);
        this.#dependencyCache = null;
    }
    /**
     * Installs all dependencies in the project.
     * @param options Options for the installation.
     * @param options.timeout The maximum time in milliseconds to wait for the command to complete.
     * @param options.force If true, forces a clean install, potentially overwriting existing modules.
     * @param options.registry The registry to use for the installation.
     * @param options.ignoreScripts If true, prevents lifecycle scripts from being executed.
     * @returns A promise that resolves when the command is complete.
     */
    async install(options = { ignoreScripts: true }) {
        const flags = [
            options.force ? this.descriptor.forceFlag : '',
            options.ignoreScripts ? this.descriptor.ignoreScriptsFlag : '',
        ].filter((flag) => flag);
        const args = [...this.descriptor.installCommand, ...flags];
        await this.#run(args, options);
        this.#dependencyCache = null;
    }
    /**
     * Gets the version of the package manager binary.
     * @returns A promise that resolves to the trimmed version string.
     */
    async getVersion() {
        const { stdout } = await this.#run(this.descriptor.versionCommand);
        return stdout.trim();
    }
    /**
     * Gets the installed details of a package from the project's dependencies.
     * @param packageName The name of the package to check.
     * @returns A promise that resolves to the installed package details, or `null` if the package is not installed.
     */
    async getInstalledPackage(packageName) {
        const cache = await this.#populateDependencyCache();
        return cache.get(packageName) ?? null;
    }
    /**
     * Gets a map of all top-level dependencies installed in the project.
     * @returns A promise that resolves to a map of package names to their installed package details.
     */
    async getProjectDependencies() {
        const cache = await this.#populateDependencyCache();
        // Return a copy to prevent external mutations of the cache.
        return new Map(cache);
    }
    /**
     * Fetches the registry metadata for a package. This is the full metadata,
     * including all versions and distribution tags.
     * @param packageName The name of the package to fetch the metadata for.
     * @param options Options for the fetch.
     * @param options.timeout The maximum time in milliseconds to wait for the command to complete.
     * @param options.registry The registry to use for the fetch.
     * @param options.bypassCache If true, ignores the in-memory cache and fetches fresh data.
     * @returns A promise that resolves to the `PackageMetadata` object, or `null` if the package is not found.
     */
    async getRegistryMetadata(packageName, options = {}) {
        const commandArgs = [...this.descriptor.getManifestCommand, packageName];
        const formatter = this.descriptor.viewCommandFieldArgFormatter;
        if (formatter) {
            commandArgs.push(...formatter(METADATA_FIELDS));
        }
        const cacheKey = options.registry ? `${packageName}|${options.registry}` : packageName;
        return this.#fetchAndParse(commandArgs, (stdout, logger) => this.descriptor.outputParsers.getRegistryMetadata(stdout, logger), { ...options, cache: this.#metadataCache, cacheKey });
    }
    /**
     * Fetches the registry manifest for a specific version of a package.
     * The manifest is similar to the package's `package.json` file.
     * @param packageName The name of the package to fetch the manifest for.
     * @param version The version of the package to fetch the manifest for.
     * @param options Options for the fetch.
     * @param options.timeout The maximum time in milliseconds to wait for the command to complete.
     * @param options.registry The registry to use for the fetch.
     * @param options.bypassCache If true, ignores the in-memory cache and fetches fresh data.
     * @returns A promise that resolves to the `PackageManifest` object, or `null` if the package is not found.
     */
    async getPackageManifest(packageName, version, options = {}) {
        const specifier = `${packageName}@${version}`;
        const commandArgs = [...this.descriptor.getManifestCommand, specifier];
        const formatter = this.descriptor.viewCommandFieldArgFormatter;
        if (formatter) {
            commandArgs.push(...formatter(MANIFEST_FIELDS));
        }
        const cacheKey = options.registry ? `${specifier}|${options.registry}` : specifier;
        return this.#fetchAndParse(commandArgs, (stdout, logger) => this.descriptor.outputParsers.getPackageManifest(stdout, logger), { ...options, cache: this.#manifestCache, cacheKey });
    }
    /**
     * Acquires a package by installing it into a temporary directory. The caller is
     * responsible for managing the lifecycle of the temporary directory by calling
     * the returned `cleanup` function.
     *
     * @param packageName The name of the package to install.
     * @param options Options for the installation.
     * @returns A promise that resolves to an object containing the temporary path
     *   and a cleanup function.
     */
    async acquireTempPackage(packageName, options = {}) {
        const workingDirectory = await this.host.createTempDirectory();
        const cleanup = () => this.host.deleteDirectory(workingDirectory);
        // Some package managers, like yarn classic, do not write a package.json when adding a package.
        // This can cause issues with subsequent `require.resolve` calls.
        // Writing an empty package.json file beforehand prevents this.
        await this.host.writeFile((0, node_path_1.join)(workingDirectory, 'package.json'), '{}');
        const args = [this.descriptor.addCommand, packageName];
        try {
            await this.#run(args, { ...options, cwd: workingDirectory });
        }
        catch (e) {
            // If the command fails, clean up the temporary directory immediately.
            await cleanup();
            throw e;
        }
        return { workingDirectory, cleanup };
    }
}
exports.PackageManager = PackageManager;
//# sourceMappingURL=package-manager.js.map