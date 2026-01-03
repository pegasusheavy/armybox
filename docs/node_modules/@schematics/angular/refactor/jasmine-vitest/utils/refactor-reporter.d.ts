/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { logging } from '@angular-devkit/core';
import ts from '../../../third_party/github.com/Microsoft/TypeScript/lib/typescript';
import { TodoCategory } from './todo-notes';
export declare class RefactorReporter {
    private logger;
    private filesScanned;
    private filesTransformed;
    private readonly todos;
    private readonly verboseLogs;
    constructor(logger: logging.LoggerApi);
    get hasTodos(): boolean;
    incrementScannedFiles(): void;
    incrementTransformedFiles(): void;
    recordTodo(category: TodoCategory): void;
    reportTransformation(sourceFile: ts.SourceFile, node: ts.Node, message: string): void;
    printSummary(verbose?: boolean): void;
}
