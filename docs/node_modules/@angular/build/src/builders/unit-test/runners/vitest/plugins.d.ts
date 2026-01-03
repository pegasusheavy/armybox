/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import type { BrowserConfigOptions, UserWorkspaceConfig, VitestPlugin } from 'vitest/node';
import type { ResultFile } from '../../../application/results';
import type { NormalizedUnitTestBuilderOptions } from '../../options';
type VitestPlugins = Awaited<ReturnType<typeof VitestPlugin>>;
interface PluginOptions {
    workspaceRoot: string;
    projectSourceRoot: string;
    projectName: string;
    buildResultFiles: ReadonlyMap<string, ResultFile>;
    testFileToEntryPoint: ReadonlyMap<string, string>;
}
interface VitestConfigPluginOptions {
    browser: BrowserConfigOptions | undefined;
    coverage: NormalizedUnitTestBuilderOptions['coverage'];
    projectName: string;
    projectSourceRoot: string;
    reporters?: string[] | [string, object][];
    setupFiles: string[];
    projectPlugins: Exclude<UserWorkspaceConfig['plugins'], undefined>;
    include: string[];
    optimizeDepsInclude: string[];
}
export declare function createVitestConfigPlugin(options: VitestConfigPluginOptions): Promise<VitestPlugins[0]>;
export declare function createVitestPlugins(pluginOptions: PluginOptions): VitestPlugins;
export {};
