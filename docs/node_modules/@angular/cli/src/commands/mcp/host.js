"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.LocalWorkspaceHost = exports.CommandError = void 0;
/**
 * @fileoverview
 * This file defines an abstraction layer for operating-system or file-system operations, such as
 * command execution. This allows for easier testing by enabling the injection of mock or
 * test-specific implementations.
 */
const fs_1 = require("fs");
const node_child_process_1 = require("node:child_process");
const promises_1 = require("node:fs/promises");
/**
 * An error thrown when a command fails to execute.
 */
class CommandError extends Error {
    stdout;
    stderr;
    code;
    constructor(message, stdout, stderr, code) {
        super(message);
        this.stdout = stdout;
        this.stderr = stderr;
        this.code = code;
    }
}
exports.CommandError = CommandError;
/**
 * A concrete implementation of the `Host` interface that runs on a local workspace.
 */
exports.LocalWorkspaceHost = {
    stat: promises_1.stat,
    existsSync: fs_1.existsSync,
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
                    reject(new CommandError(message, stdout, stderr, code));
                }
            });
            childProcess.on('error', (err) => {
                if (err.name === 'AbortError') {
                    const message = `Process timed out.`;
                    reject(new CommandError(message, stdout, stderr, null));
                    return;
                }
                const message = `Process failed with error: ${err.message}`;
                reject(new CommandError(message, stdout, stderr, null));
            });
        });
    },
};
//# sourceMappingURL=host.js.map