/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST, TmplAstForLoopBlock } from '@angular/compiler';
import ts from 'typescript';
import { TcbExpressionTranslator } from './expression';
import type { Context } from './context';
import type { Scope } from './scope';
import { TcbOp } from './base';
/**
 * A `TcbOp` which renders a `for` block as a TypeScript `for...of` loop.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbForOfOp extends TcbOp {
    private tcb;
    private scope;
    private block;
    constructor(tcb: Context, scope: Scope, block: TmplAstForLoopBlock);
    get optional(): boolean;
    execute(): null;
}
export declare class TcbForLoopTrackTranslator extends TcbExpressionTranslator {
    private block;
    private allowedVariables;
    constructor(tcb: Context, scope: Scope, block: TmplAstForLoopBlock);
    protected resolve(ast: AST): ts.Expression | null;
}
