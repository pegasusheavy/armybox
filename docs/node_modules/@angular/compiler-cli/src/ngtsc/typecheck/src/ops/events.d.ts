/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST, DirectiveOwner, TmplAstBoundAttribute, TmplAstBoundEvent } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
import { TypeCheckableDirectiveMeta } from '../../api';
import { LocalSymbol } from './references';
/**
 * Similar to `tcbExpression`, this function converts the provided `AST` expression into a
 * `ts.Expression`, with special handling of the `$event` variable that can be used within event
 * bindings.
 */
export declare function tcbEventHandlerExpression(ast: AST, tcb: Context, scope: Scope): ts.Expression;
/**
 * A `TcbOp` which generates code to check event bindings on an element that correspond with the
 * outputs of a directive.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbDirectiveOutputsOp extends TcbOp {
    private tcb;
    private scope;
    private node;
    private inputs;
    private outputs;
    private dir;
    constructor(tcb: Context, scope: Scope, node: DirectiveOwner, inputs: TmplAstBoundAttribute[] | null, outputs: TmplAstBoundEvent[], dir: TypeCheckableDirectiveMeta);
    get optional(): boolean;
    execute(): null;
}
/**
 * A `TcbOp` which generates code to check "unclaimed outputs" - event bindings on an element which
 * were not attributed to any directive or component, and are instead processed against the HTML
 * element itself.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbUnclaimedOutputsOp extends TcbOp {
    private tcb;
    private scope;
    private target;
    private outputs;
    private inputs;
    private claimedOutputs;
    constructor(tcb: Context, scope: Scope, target: LocalSymbol, outputs: TmplAstBoundEvent[], inputs: TmplAstBoundAttribute[] | null, claimedOutputs: Set<string> | null);
    get optional(): boolean;
    execute(): null;
}
