/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { Stats } from 'node:fs';
import { Host } from '../host';
/**
 * A mock `Host` implementation for testing.
 * This class allows for simulating a file system in memory.
 */
export declare class MockHost implements Host {
    private readonly fs;
    constructor(files?: Record<string, string[] | true>);
    stat(path: string): Promise<Stats>;
    readdir(path: string): Promise<string[]>;
    runCommand(): Promise<{
        stdout: string;
        stderr: string;
    }>;
    createTempDirectory(): Promise<string>;
    deleteDirectory(): Promise<void>;
    writeFile(): Promise<void>;
}
