/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import type { BrowserConfigOptions } from 'vitest/node';
export interface BrowserConfiguration {
    browser?: BrowserConfigOptions;
    errors?: string[];
}
export declare function setupBrowserConfiguration(browsers: string[] | undefined, debug: boolean, projectSourceRoot: string, viewport: {
    width: number;
    height: number;
} | undefined): Promise<BrowserConfiguration>;
