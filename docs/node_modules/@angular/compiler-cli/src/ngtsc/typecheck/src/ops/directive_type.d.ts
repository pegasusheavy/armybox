/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { DirectiveOwner } from '@angular/compiler';
import ts from 'typescript';
import type { Context } from './context';
import type { Scope } from './scope';
import { TcbOp } from './base';
import { TypeCheckableDirectiveMeta } from '../../api';
/**
 * A `TcbOp` which constructs an instance of a directive. For generic directives, generic
 * parameters are set to `any` type.
 */
export declare abstract class TcbDirectiveTypeOpBase extends TcbOp {
    protected tcb: Context;
    protected scope: Scope;
    protected node: DirectiveOwner;
    protected dir: TypeCheckableDirectiveMeta;
    constructor(tcb: Context, scope: Scope, node: DirectiveOwner, dir: TypeCheckableDirectiveMeta);
    get optional(): boolean;
    execute(): ts.Identifier;
}
/**
 * A `TcbOp` which constructs an instance of a non-generic directive _without_ setting any of its
 * inputs. Inputs are later set in the `TcbDirectiveInputsOp`. Type checking was found to be
 * faster when done in this way as opposed to `TcbDirectiveCtorOp` which is only necessary when the
 * directive is generic.
 *
 * Executing this operation returns a reference to the directive instance variable with its inferred
 * type.
 */
export declare class TcbNonGenericDirectiveTypeOp extends TcbDirectiveTypeOpBase {
    /**
     * Creates a variable declaration for this op's directive of the argument type. Returns the id of
     * the newly created variable.
     */
    execute(): ts.Identifier;
}
/**
 * A `TcbOp` which constructs an instance of a generic directive with its generic parameters set
 * to `any` type. This op is like `TcbDirectiveTypeOp`, except that generic parameters are set to
 * `any` type. This is used for situations where we want to avoid inlining.
 *
 * Executing this operation returns a reference to the directive instance variable with its generic
 * type parameters set to `any`.
 */
export declare class TcbGenericDirectiveTypeWithAnyParamsOp extends TcbDirectiveTypeOpBase {
    execute(): ts.Identifier;
}
