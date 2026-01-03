/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstSwitchBlock } from '@angular/compiler';
import { TcbOp } from './base';
import type { Scope } from './scope';
import type { Context } from './context';
/**
 * A `TcbOp` which renders a `switch` block as a TypeScript `switch` statement.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbSwitchOp extends TcbOp {
    private tcb;
    private scope;
    private block;
    constructor(tcb: Context, scope: Scope, block: TmplAstSwitchBlock);
    get optional(): boolean;
    execute(): null;
    private generateGuard;
}
