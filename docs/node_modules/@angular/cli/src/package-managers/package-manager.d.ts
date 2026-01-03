/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { Host } from './host';
import { Logger } from './logger';
import { PackageManagerDescriptor } from './package-manager-descriptor';
import { PackageManifest, PackageMetadata } from './package-metadata';
import { InstalledPackage } from './package-tree';
/**
 * Options to configure the `PackageManager` instance.
 */
export interface PackageManagerOptions {
    /**
     * If true, no commands will be executed, but they will be logged to the logger.
     * A logger must be provided if this is true.
     */
    dryRun?: boolean;
    /** A logger instance for debugging and dry run output. */
    logger?: Logger;
}
/**
 * A class that provides a high-level, package-manager-agnostic API for
 * interacting with a project's dependencies.
 *
 * This class is an implementation of the Strategy design pattern. It is
 * instantiated with a `PackageManagerDescriptor` that defines the specific
 * commands and flags for a given package manager.
 */
export declare class PackageManager {
    #private;
    private readonly host;
    private readonly cwd;
    private readonly descriptor;
    private readonly options;
    /**
     * Creates a new `PackageManager` instance.
     * @param host A `Host` instance for interacting with the file system and running commands.
     * @param cwd The absolute path to the project's working directory.
     * @param descriptor A `PackageManagerDescriptor` that defines the commands for a specific package manager.
     * @param options An options object to configure the instance.
     */
    constructor(host: Host, cwd: string, descriptor: PackageManagerDescriptor, options?: PackageManagerOptions);
    /**
     * The name of the package manager's binary.
     */
    get name(): string;
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
    add(packageName: string, save: 'exact' | 'tilde' | 'none', asDevDependency: boolean, noLockfile: boolean, ignoreScripts: boolean, options?: {
        registry?: string;
    }): Promise<void>;
    /**
     * Installs all dependencies in the project.
     * @param options Options for the installation.
     * @param options.timeout The maximum time in milliseconds to wait for the command to complete.
     * @param options.force If true, forces a clean install, potentially overwriting existing modules.
     * @param options.registry The registry to use for the installation.
     * @param options.ignoreScripts If true, prevents lifecycle scripts from being executed.
     * @returns A promise that resolves when the command is complete.
     */
    install(options?: {
        timeout?: number;
        force?: boolean;
        registry?: string;
        ignoreScripts?: boolean;
    }): Promise<void>;
    /**
     * Gets the version of the package manager binary.
     * @returns A promise that resolves to the trimmed version string.
     */
    getVersion(): Promise<string>;
    /**
     * Gets the installed details of a package from the project's dependencies.
     * @param packageName The name of the package to check.
     * @returns A promise that resolves to the installed package details, or `null` if the package is not installed.
     */
    getInstalledPackage(packageName: string): Promise<InstalledPackage | null>;
    /**
     * Gets a map of all top-level dependencies installed in the project.
     * @returns A promise that resolves to a map of package names to their installed package details.
     */
    getProjectDependencies(): Promise<Map<string, InstalledPackage>>;
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
    getRegistryMetadata(packageName: string, options?: {
        timeout?: number;
        registry?: string;
        bypassCache?: boolean;
    }): Promise<PackageMetadata | null>;
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
    getPackageManifest(packageName: string, version: string, options?: {
        timeout?: number;
        registry?: string;
        bypassCache?: boolean;
    }): Promise<PackageManifest | null>;
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
    acquireTempPackage(packageName: string, options?: {
        registry?: string;
    }): Promise<{
        workingDirectory: string;
        cleanup: () => Promise<void>;
    }>;
}
