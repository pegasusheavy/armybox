"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.RefactorReporter = void 0;
const typescript_1 = __importDefault(require("../../../third_party/github.com/Microsoft/TypeScript/lib/typescript"));
class RefactorReporter {
    logger;
    filesScanned = 0;
    filesTransformed = 0;
    todos = new Map();
    verboseLogs = new Map();
    constructor(logger) {
        this.logger = logger;
    }
    get hasTodos() {
        return this.todos.size > 0;
    }
    incrementScannedFiles() {
        this.filesScanned++;
    }
    incrementTransformedFiles() {
        this.filesTransformed++;
    }
    recordTodo(category) {
        this.todos.set(category, (this.todos.get(category) ?? 0) + 1);
    }
    reportTransformation(sourceFile, node, message) {
        const { line } = typescript_1.default.getLineAndCharacterOfPosition(sourceFile, typescript_1.default.getOriginalNode(node).getStart());
        const filePath = sourceFile.fileName;
        let logs = this.verboseLogs.get(filePath);
        if (!logs) {
            logs = [];
            this.verboseLogs.set(filePath, logs);
        }
        logs.push(`L${line + 1}: ${message}`);
    }
    printSummary(verbose = false) {
        if (verbose && this.verboseLogs.size > 0) {
            this.logger.info('Detailed Transformation Log:');
            for (const [filePath, logs] of this.verboseLogs) {
                this.logger.info(`Processing: ${filePath}`);
                logs.forEach((log) => this.logger.info(`  - ${log}`));
            }
            this.logger.info(''); // Add a blank line for separation
        }
        this.logger.info('Jasmine to Vitest Refactoring Summary:');
        this.logger.info(`- ${this.filesScanned} test file(s) scanned.`);
        this.logger.info(`- ${this.filesTransformed} file(s) transformed.`);
        const filesSkipped = this.filesScanned - this.filesTransformed;
        if (filesSkipped > 0) {
            this.logger.info(`- ${filesSkipped} file(s) skipped (no changes needed).`);
        }
        if (this.todos.size > 0) {
            const totalTodos = [...this.todos.values()].reduce((a, b) => a + b, 0);
            this.logger.warn(`- ${totalTodos} TODO(s) added for manual review:`);
            for (const [category, count] of this.todos) {
                this.logger.warn(`  - ${count}x ${category}`);
            }
        }
    }
}
exports.RefactorReporter = RefactorReporter;
//# sourceMappingURL=refactor-reporter.js.map