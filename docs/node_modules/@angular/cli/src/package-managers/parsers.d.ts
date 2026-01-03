/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
/**
 * @fileoverview This file contains the parser functions that are used to
 * interpret the output of various package manager commands. Separating these
 * into their own file improves modularity and allows for focused testing.
 */
import { Logger } from './logger';
import { PackageManifest, PackageMetadata } from './package-metadata';
import { InstalledPackage } from './package-tree';
/**
 * Parses the output of `npm list` or a compatible command.
 *
 * The expected JSON structure is:
 * ```json
 * {
 *   "dependencies": {
 *     "@angular/cli": {
 *       "version": "18.0.0",
 *       "path": "/path/to/project/node_modules/@angular/cli", // path is optional
 *       ... (other package.json properties)
 *     }
 *   }
 * }
 * ```
 *
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns A map of package names to their installed package details.
 */
export declare function parseNpmLikeDependencies(stdout: string, logger?: Logger): Map<string, InstalledPackage>;
/**
 * Parses the output of `yarn list` (classic).
 *
 * The expected output is a JSON stream (JSONL), where each line is a JSON object.
 * The relevant object has a `type` of `'tree'`.
 * Yarn classic does not provide a path, so the `path` property will be `undefined`.
 *
 * ```json
 * {"type":"tree","data":{"trees":[{"name":"@angular/cli@18.0.0","children":[]}]}}
 * ```
 *
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns A map of package names to their installed package details.
 */
export declare function parseYarnClassicDependencies(stdout: string, logger?: Logger): Map<string, InstalledPackage>;
/**
 * Parses the output of `yarn list` (modern).
 *
 * The expected JSON structure is a single object.
 * Yarn modern does not provide a path, so the `path` property will be `undefined`.
 *
 * ```json
 * {
 *   "trees": [
 *     { "name": "@angular/cli@18.0.0", "children": [] }
 *   ]
 * }
 * ```
 *
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns A map of package names to their installed package details.
 */
export declare function parseYarnModernDependencies(stdout: string, logger?: Logger): Map<string, InstalledPackage>;
/**
 * Parses the output of `npm view` or a compatible command to get a package manifest.
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns The package manifest object.
 */
export declare function parseNpmLikeManifest(stdout: string, logger?: Logger): PackageManifest | null;
/**
 * Parses the output of `npm view` or a compatible command to get package metadata.
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns The package metadata object.
 */
export declare function parseNpmLikeMetadata(stdout: string, logger?: Logger): PackageMetadata | null;
/**
 * Parses the output of `yarn info` (classic).
 * @param stdout The standard output of the command.
 * @param logger An optional logger instance.
 * @returns The package manifest object.
 */
export declare function parseYarnLegacyManifest(stdout: string, logger?: Logger): PackageManifest | null;
