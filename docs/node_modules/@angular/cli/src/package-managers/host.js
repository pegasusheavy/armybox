"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.NodeJS_HOST = void 0;
/**
 * @fileoverview
 * This file defines an abstraction layer for side-effectful operations, such as
 * file system access and command execution. This allows for easier testing by
 * enabling the injection of mock or test-specific implementations.
 */
const node_child_process_1 = require("node:child_process");
const promises_1 = require("node:fs/promises");
const node_os_1 = require("node:os");
const node_path_1 = require("node:path");
const error_1 = require("./error");
/**
 * A concrete implementation of the `Host` interface that uses the Node.js APIs.
 */
exports.NodeJS_HOST = {
    stat: promises_1.stat,
    readdir: promises_1.readdir,
    writeFile: promises_1.writeFile,
    createTempDirectory: () => (0, promises_1.mkdtemp)((0, node_path_1.join)((0, node_os_1.tmpdir)(), 'angular-cli-')),
    deleteDirectory: (path) => (0, promises_1.rm)(path, { recursive: true, force: true }),
    runCommand: async (command, args, options = {}) => {
        const signal = options.timeout ? AbortSignal.timeout(options.timeout) : undefined;
        return new Promise((resolve, reject) => {
            const childProcess = (0, node_child_process_1.spawn)(command, args, {
                shell: false,
                stdio: options.stdio ?? 'pipe',
                signal,
                cwd: options.cwd,
                env: {
                    ...process.env,
                    ...options.env,
                },
            });
            let stdout = '';
            childProcess.stdout?.on('data', (data) => (stdout += data.toString()));
            let stderr = '';
            childProcess.stderr?.on('data', (data) => (stderr += data.toString()));
            childProcess.on('close', (code) => {
                if (code === 0) {
                    resolve({ stdout, stderr });
                }
                else {
                    const message = `Process exited with code ${code}.`;
                    reject(new error_1.PackageManagerError(message, stdout, stderr, code));
                }
            });
            childProcess.on('error', (err) => {
                if (err.name === 'AbortError') {
                    const message = `Process timed out.`;
                    reject(new error_1.PackageManagerError(message, stdout, stderr, null));
                    return;
                }
                const message = `Process failed with error: ${err.message}`;
                reject(new error_1.PackageManagerError(message, stdout, stderr, null));
            });
        });
    },
};
//# sourceMappingURL=host.js.map