/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TcbOp } from './base';
import type { Scope } from './scope';
/**
 * A `TcbOp` which generates a completion point for the component context.
 *
 * This completion point looks like `this. ;` in the TCB output, and does not produce diagnostics.
 * TypeScript autocompletion APIs can be used at this completion point (after the '.') to produce
 * autocompletion results of properties and methods from the template's component context.
 */
export declare class TcbComponentContextCompletionOp extends TcbOp {
    private scope;
    constructor(scope: Scope);
    readonly optional = false;
    execute(): null;
}
