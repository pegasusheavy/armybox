/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstComponent, TmplAstElement, TmplAstHostElement } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import { Context } from './context';
/**
 * A `TcbOp` which feeds elements and unclaimed properties to the `DomSchemaChecker`.
 *
 * The DOM schema is not checked via TCB code generation. Instead, the `DomSchemaChecker` ingests
 * elements and property bindings and accumulates synthetic `ts.Diagnostic`s out-of-band. These are
 * later merged with the diagnostics generated from the TCB.
 *
 * For convenience, the TCB iteration of the template is used to drive the `DomSchemaChecker` via
 * the `TcbDomSchemaCheckerOp`.
 */
export declare class TcbDomSchemaCheckerOp extends TcbOp {
    private tcb;
    private element;
    private checkElement;
    private claimedInputs;
    constructor(tcb: Context, element: TmplAstElement | TmplAstComponent | TmplAstHostElement, checkElement: boolean, claimedInputs: Set<string> | null);
    get optional(): boolean;
    execute(): ts.Expression | null;
    private getTagName;
}
