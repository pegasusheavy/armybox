"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.MockHost = void 0;
/**
 * A mock `Host` implementation for testing.
 * This class allows for simulating a file system in memory.
 */
class MockHost {
    fs = new Map();
    constructor(files = {}) {
        // Normalize paths to use forward slashes for consistency in tests.
        for (const [path, content] of Object.entries(files)) {
            this.fs.set(path.replace(/\\/g, '/'), content);
        }
    }
    stat(path) {
        const content = this.fs.get(path.replace(/\\/g, '/'));
        if (content === undefined) {
            return Promise.reject(new Error(`File not found: ${path}`));
        }
        // A `true` value signifies a directory in our mock file system.
        return Promise.resolve({ isDirectory: () => content === true });
    }
    readdir(path) {
        const content = this.fs.get(path.replace(/\\/g, '/'));
        if (content === true || content === undefined) {
            // This should be a directory with a file list.
            return Promise.reject(new Error(`Directory not found or not a directory: ${path}`));
        }
        return Promise.resolve(content);
    }
    runCommand() {
        throw new Error('Method not implemented.');
    }
    createTempDirectory() {
        throw new Error('Method not implemented.');
    }
    deleteDirectory() {
        throw new Error('Method not implemented.');
    }
    writeFile() {
        throw new Error('Method not implemented.');
    }
}
exports.MockHost = MockHost;
//# sourceMappingURL=mock-host.js.map