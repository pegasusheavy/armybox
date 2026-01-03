/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST, PropertyRead, TmplAstLetDeclaration } from '@angular/compiler';
import ts from 'typescript';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
/**
 * Process an `AST` expression and convert it into a `ts.Expression`, generating references to the
 * correct identifiers in the current scope.
 */
export declare function tcbExpression(ast: AST, tcb: Context, scope: Scope): ts.Expression;
/**
 * Wraps an expression in an `unwrapSignal` call which extracts the signal's value.
 */
export declare function unwrapWritableSignal(expression: ts.Expression, tcb: Context): ts.CallExpression;
/**
 * A `TcbOp` which renders an Angular expression (e.g. `{{foo() && bar.baz}}`).
 *
 * Executing this operation returns nothing.
 */
export declare class TcbExpressionOp extends TcbOp {
    private tcb;
    private scope;
    private expression;
    constructor(tcb: Context, scope: Scope, expression: AST);
    get optional(): boolean;
    execute(): null;
}
export declare class TcbExpressionTranslator {
    protected tcb: Context;
    protected scope: Scope;
    constructor(tcb: Context, scope: Scope);
    translate(ast: AST): ts.Expression;
    /**
     * Resolve an `AST` expression within the given scope.
     *
     * Some `AST` expressions refer to top-level concepts (references, variables, the component
     * context). This method assists in resolving those.
     */
    protected resolve(ast: AST): ts.Expression | null;
    private getTargetNodeExpression;
    protected isValidLetDeclarationAccess(target: TmplAstLetDeclaration, ast: PropertyRead): boolean;
}
