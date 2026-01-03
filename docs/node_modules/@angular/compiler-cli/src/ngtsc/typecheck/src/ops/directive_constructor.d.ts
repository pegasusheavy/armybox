/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { DirectiveOwner } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import { Context } from './context';
import type { Scope } from './scope';
import { TypeCheckableDirectiveMeta } from '../../api';
import { CustomFieldType } from './signal_forms';
/**
 * A `TcbOp` which constructs an instance of a directive with types inferred from its inputs. The
 * inputs themselves are not checked here; checking of inputs is achieved in `TcbDirectiveInputsOp`.
 * Any errors reported in this statement are ignored, as the type constructor call is only present
 * for type-inference.
 *
 * When a Directive is generic, it is required that the TCB generates the instance using this method
 * in order to infer the type information correctly.
 *
 * Executing this operation returns a reference to the directive instance variable with its inferred
 * type.
 */
export declare class TcbDirectiveCtorOp extends TcbOp {
    private tcb;
    private scope;
    private node;
    private dir;
    private customControlType;
    constructor(tcb: Context, scope: Scope, node: DirectiveOwner, dir: TypeCheckableDirectiveMeta, customControlType: CustomFieldType | null);
    get optional(): boolean;
    execute(): ts.Identifier;
    circularFallback(): TcbOp;
}
/**
 * A `TcbOp` which is used to generate a fallback expression if the inference of a directive type
 * via `TcbDirectiveCtorOp` requires a reference to its own type. This can happen using a template
 * reference:
 *
 * ```html
 * <some-cmp #ref [prop]="ref.foo"></some-cmp>
 * ```
 *
 * In this case, `TcbDirectiveCtorCircularFallbackOp` will add a second inference of the directive
 * type to the type-check block, this time calling the directive's type constructor without any
 * input expressions. This infers the widest possible supertype for the directive, which is used to
 * resolve any recursive references required to infer the real type.
 */
export declare class TcbDirectiveCtorCircularFallbackOp extends TcbOp {
    private tcb;
    private scope;
    private dir;
    constructor(tcb: Context, scope: Scope, dir: TypeCheckableDirectiveMeta);
    get optional(): boolean;
    execute(): ts.Identifier;
}
