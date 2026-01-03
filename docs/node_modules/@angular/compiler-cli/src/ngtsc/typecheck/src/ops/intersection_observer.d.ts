/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST } from '@angular/compiler';
import { TcbOp } from './base';
import { Context } from './context';
import type { Scope } from './scope';
/**
 * A `TcbOp` which can be used to type check the options of an `IntersectionObserver`.
 */
export declare class TcbIntersectionObserverOp extends TcbOp {
    private tcb;
    private scope;
    private options;
    constructor(tcb: Context, scope: Scope, options: AST);
    readonly optional = false;
    execute(): null;
}
