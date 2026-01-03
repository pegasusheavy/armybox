/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstIfBlock } from '@angular/compiler';
import { TcbOp } from './base';
import type { Scope } from './scope';
import type { Context } from './context';
/**
 * A `TcbOp` which renders an `if` template block as a TypeScript `if` statement.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbIfOp extends TcbOp {
    private tcb;
    private scope;
    private block;
    private expressionScopes;
    constructor(tcb: Context, scope: Scope, block: TmplAstIfBlock);
    get optional(): boolean;
    execute(): null;
    private generateBranch;
    private getBranchScope;
    private generateBranchGuard;
}
