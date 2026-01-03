/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstComponent, TmplAstElement } from '@angular/compiler';
import { TcbOp } from './base';
import { Context } from './context';
/**
 * A `TcbOp` that finds and flags control flow nodes that interfere with content projection.
 *
 * Context:
 * Control flow blocks try to emulate the content projection behavior of `*ngIf` and `*ngFor`
 * in order to reduce breakages when moving from one syntax to the other (see #52414), however the
 * approach only works if there's only one element at the root of the control flow expression.
 * This means that a stray sibling node (e.g. text) can prevent an element from being projected
 * into the right slot. The purpose of the `TcbOp` is to find any places where a node at the root
 * of a control flow expression *would have been projected* into a specific slot, if the control
 * flow node didn't exist.
 */
export declare class TcbControlFlowContentProjectionOp extends TcbOp {
    private tcb;
    private element;
    private ngContentSelectors;
    private componentName;
    private readonly category;
    constructor(tcb: Context, element: TmplAstElement | TmplAstComponent, ngContentSelectors: string[], componentName: string);
    readonly optional = false;
    execute(): null;
    private findPotentialControlFlowNodes;
    private shouldCheck;
}
