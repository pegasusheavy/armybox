/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstHostElement } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
/**
 * A `TcbOp` which creates an expression for a the host element of a directive.
 *
 * Executing this operation returns a reference to the element variable.
 */
export declare class TcbHostElementOp extends TcbOp {
    private tcb;
    private scope;
    private element;
    readonly optional = true;
    constructor(tcb: Context, scope: Scope, element: TmplAstHostElement);
    execute(): ts.Identifier;
}
