/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstComponent, TmplAstDirective, TmplAstElement, TmplAstHostElement, TmplAstLetDeclaration, TmplAstReference, TmplAstTemplate, TmplAstVariable } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
import { TypeCheckableDirectiveMeta } from '../../api';
/** Types that can referenced locally in a template. */
export type LocalSymbol = TmplAstElement | TmplAstTemplate | TmplAstVariable | TmplAstLetDeclaration | TmplAstReference | TmplAstHostElement | TmplAstComponent | TmplAstDirective;
/**
 * A `TcbOp` which creates a variable for a local ref in a template.
 * The initializer for the variable is the variable expression for the directive, template, or
 * element the ref refers to. When the reference is used in the template, those TCB statements will
 * access this variable as well. For example:
 * ```ts
 * var _t1 = document.createElement('div');
 * var _t2 = _t1;
 * _t2.value
 * ```
 * This operation supports more fluent lookups for the `TemplateTypeChecker` when getting a symbol
 * for a reference. In most cases, this isn't essential; that is, the information for the symbol
 * could be gathered without this operation using the `BoundTarget`. However, for the case of
 * ng-template references, we will need this reference variable to not only provide a location in
 * the shim file, but also to narrow the variable to the correct `TemplateRef<T>` type rather than
 * `TemplateRef<any>` (this work is still TODO).
 *
 * Executing this operation returns a reference to the directive instance variable with its inferred
 * type.
 */
export declare class TcbReferenceOp extends TcbOp {
    private readonly tcb;
    private readonly scope;
    private readonly node;
    private readonly host;
    private readonly target;
    constructor(tcb: Context, scope: Scope, node: TmplAstReference, host: TmplAstElement | TmplAstTemplate | TmplAstComponent | TmplAstDirective, target: TypeCheckableDirectiveMeta | TmplAstTemplate | TmplAstElement);
    readonly optional = true;
    execute(): ts.Identifier;
}
/**
 * A `TcbOp` which is used when the target of a reference is missing. This operation generates a
 * variable of type any for usages of the invalid reference to resolve to. The invalid reference
 * itself is recorded out-of-band.
 */
export declare class TcbInvalidReferenceOp extends TcbOp {
    private readonly tcb;
    private readonly scope;
    constructor(tcb: Context, scope: Scope);
    readonly optional = true;
    execute(): ts.Identifier;
}
