/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstLetDeclaration } from '@angular/compiler';
import ts from 'typescript';
import { Context } from './context';
import type { Scope } from './scope';
import { TcbOp } from './base';
/**
 * A `TcbOp` which generates a constant for a `TmplAstLetDeclaration`.
 *
 * Executing this operation returns a reference to the `@let` declaration.
 */
export declare class TcbLetDeclarationOp extends TcbOp {
    private tcb;
    private scope;
    private node;
    constructor(tcb: Context, scope: Scope, node: TmplAstLetDeclaration);
    /**
     * `@let` declarations are mandatory, because their expressions
     * should be checked even if they aren't referenced anywhere.
     */
    readonly optional = false;
    execute(): ts.Identifier;
}
