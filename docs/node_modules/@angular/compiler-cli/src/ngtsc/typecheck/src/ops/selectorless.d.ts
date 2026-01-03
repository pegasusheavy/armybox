/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstComponent } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import { Context } from './context';
import type { Scope } from './scope';
export declare function getComponentTagName(node: TmplAstComponent): string;
/**
 * A `TcbOp` which creates an expression for a native DOM element from a `TmplAstComponent`.
 *
 * Executing this operation returns a reference to the element variable.
 */
export declare class TcbComponentNodeOp extends TcbOp {
    private tcb;
    private scope;
    private component;
    readonly optional = true;
    constructor(tcb: Context, scope: Scope, component: TmplAstComponent);
    execute(): ts.Identifier;
}
