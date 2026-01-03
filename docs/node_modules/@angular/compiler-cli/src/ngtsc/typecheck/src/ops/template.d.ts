/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstTemplate } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
/**
 * A `TcbOp` which generates a variable for a `TmplAstTemplate`'s context.
 *
 * Executing this operation returns a reference to the template's context variable.
 */
export declare class TcbTemplateContextOp extends TcbOp {
    private tcb;
    private scope;
    constructor(tcb: Context, scope: Scope);
    readonly optional = true;
    execute(): ts.Identifier;
}
/**
 * A `TcbOp` which descends into a `TmplAstTemplate`'s children and generates type-checking code for
 * them.
 *
 * This operation wraps the children's type-checking code in an `if` block, which may include one
 * or more type guard conditions that narrow types within the template body.
 */
export declare class TcbTemplateBodyOp extends TcbOp {
    private tcb;
    private scope;
    private template;
    constructor(tcb: Context, scope: Scope, template: TmplAstTemplate);
    get optional(): boolean;
    execute(): null;
    private addDirectiveGuards;
}
