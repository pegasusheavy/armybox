/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST, TmplAstBoundAttribute, TmplAstComponent, TmplAstDirective, TmplAstElement, TmplAstTemplate } from '@angular/compiler';
import ts from 'typescript';
import type { Context } from './context';
import type { Scope } from './scope';
import { TypeCheckableDirectiveMeta } from '../../api';
import { TcbOp } from './base';
import { CustomFieldType } from './signal_forms';
import { LocalSymbol } from './references';
/**
 * Translates the given attribute binding to a `ts.Expression`.
 */
export declare function translateInput(value: AST | string, tcb: Context, scope: Scope): ts.Expression;
/**
 * A `TcbOp` which generates code to check input bindings on an element that correspond with the
 * members of a directive.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbDirectiveInputsOp extends TcbOp {
    private tcb;
    private scope;
    private node;
    private dir;
    private customControlType;
    constructor(tcb: Context, scope: Scope, node: TmplAstTemplate | TmplAstElement | TmplAstComponent | TmplAstDirective, dir: TypeCheckableDirectiveMeta, customControlType: CustomFieldType | null);
    get optional(): boolean;
    execute(): null;
    private checkRequiredInputs;
}
/**
 * A `TcbOp` which generates code to check "unclaimed inputs" - bindings on an element which were
 * not attributed to any directive or component, and are instead processed against the HTML element
 * itself.
 *
 * Currently, only the expressions of these bindings are checked. The targets of the bindings are
 * checked against the DOM schema via a `TcbDomSchemaCheckerOp`.
 *
 * Executing this operation returns nothing.
 */
export declare class TcbUnclaimedInputsOp extends TcbOp {
    private tcb;
    private scope;
    private inputs;
    private target;
    private claimedInputs;
    constructor(tcb: Context, scope: Scope, inputs: TmplAstBoundAttribute[], target: LocalSymbol, claimedInputs: Set<string> | null);
    get optional(): boolean;
    execute(): null;
}
