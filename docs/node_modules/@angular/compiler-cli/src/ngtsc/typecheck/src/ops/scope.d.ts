/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { TmplAstForLoopBlock, TmplAstHostElement, TmplAstIfBlockBranch, TmplAstLetDeclaration, TmplAstNode, TmplAstReference, TmplAstTemplate, TmplAstVariable } from '@angular/compiler';
import ts from 'typescript';
import { TypeCheckableDirectiveMeta } from '../../api';
import { Context } from './context';
import { LocalSymbol } from './references';
/**
 * Local scope within the type check block for a particular template.
 *
 * The top-level template and each nested `<ng-template>` have their own `Scope`, which exist in a
 * hierarchy. The structure of this hierarchy mirrors the syntactic scopes in the generated type
 * check block, where each nested template is encased in an `if` structure.
 *
 * As a template's `TcbOp`s are executed in a given `Scope`, statements are added via
 * `addStatement()`. When this processing is complete, the `Scope` can be turned into a `ts.Block`
 * via `renderToBlock()`.
 *
 * If a `TcbOp` requires the output of another, it can call `resolve()`.
 */
export declare class Scope {
    private tcb;
    private parent;
    private guard;
    /**
     * A queue of operations which need to be performed to generate the TCB code for this scope.
     *
     * This array can contain either a `TcbOp` which has yet to be executed, or a `ts.Expression|null`
     * representing the memoized result of executing the operation. As operations are executed, their
     * results are written into the `opQueue`, overwriting the original operation.
     *
     * If an operation is in the process of being executed, it is temporarily overwritten here with
     * `INFER_TYPE_FOR_CIRCULAR_OP_EXPR`. This way, if a cycle is encountered where an operation
     * depends transitively on its own result, the inner operation will infer the least narrow type
     * that fits instead. This has the same semantics as TypeScript itself when types are referenced
     * circularly.
     */
    private opQueue;
    /**
     * A map of `TmplAstElement`s to the index of their `TcbElementOp` in the `opQueue`
     */
    private elementOpMap;
    /**
     * A map of `TmplAstHostElement`s to the index of their `TcbHostElementOp` in the `opQueue`
     */
    private hostElementOpMap;
    /**
     * A map of `TmplAstComponent`s to the index of their `TcbComponentNodeOp` in the `opQueue`
     */
    private componentNodeOpMap;
    /**
     * A map of maps which tracks the index of `TcbDirectiveCtorOp`s in the `opQueue` for each
     * directive on a `TmplAstElement` or `TmplAstTemplate` node.
     */
    private directiveOpMap;
    /**
     * A map of `TmplAstReference`s to the index of their `TcbReferenceOp` in the `opQueue`
     */
    private referenceOpMap;
    /**
     * Map of immediately nested <ng-template>s (within this `Scope`) represented by `TmplAstTemplate`
     * nodes to the index of their `TcbTemplateContextOp`s in the `opQueue`.
     */
    private templateCtxOpMap;
    /**
     * Map of variables declared on the template that created this `Scope` (represented by
     * `TmplAstVariable` nodes) to the index of their `TcbVariableOp`s in the `opQueue`, or to
     * pre-resolved variable identifiers.
     */
    private varMap;
    /**
     * A map of the names of `TmplAstLetDeclaration`s to the index of their op in the `opQueue`.
     *
     * Assumes that there won't be duplicated `@let` declarations within the same scope.
     */
    private letDeclOpMap;
    /**
     * Statements for this template.
     *
     * Executing the `TcbOp`s in the `opQueue` populates this array.
     */
    private statements;
    /**
     * Gets names of the for loop context variables and their types.
     */
    private static getForLoopContextVariableTypes;
    private constructor();
    /**
     * Constructs a `Scope` given either a `TmplAstTemplate` or a list of `TmplAstNode`s.
     *
     * @param tcb the overall context of TCB generation.
     * @param parentScope the `Scope` of the parent template (if any) or `null` if this is the root
     * `Scope`.
     * @param scopedNode Node that provides the scope around the child nodes (e.g. a
     * `TmplAstTemplate` node exposing variables to its children).
     * @param children Child nodes that should be appended to the TCB.
     * @param guard an expression that is applied to this scope for type narrowing purposes.
     */
    static forNodes(tcb: Context, parentScope: Scope | null, scopedNode: TmplAstTemplate | TmplAstIfBlockBranch | TmplAstForLoopBlock | TmplAstHostElement | null, children: TmplAstNode[] | null, guard: ts.Expression | null): Scope;
    /** Registers a local variable with a scope. */
    private static registerVariable;
    /**
     * Look up a `ts.Expression` representing the value of some operation in the current `Scope`,
     * including any parent scope(s). This method always returns a mutable clone of the
     * `ts.Expression` with the comments cleared.
     *
     * @param node a `TmplAstNode` of the operation in question. The lookup performed will depend on
     * the type of this node:
     *
     * Assuming `directive` is not present, then `resolve` will return:
     *
     * * `TmplAstElement` - retrieve the expression for the element DOM node
     * * `TmplAstTemplate` - retrieve the template context variable
     * * `TmplAstVariable` - retrieve a template let- variable
     * * `TmplAstLetDeclaration` - retrieve a template `@let` declaration
     * * `TmplAstReference` - retrieve variable created for the local ref
     *
     * @param directive if present, a directive type on a `TmplAstElement` or `TmplAstTemplate` to
     * look up instead of the default for an element or template node.
     */
    resolve(node: LocalSymbol, directive?: TypeCheckableDirectiveMeta): ts.Identifier | ts.NonNullExpression;
    /**
     * Add a statement to this scope.
     */
    addStatement(stmt: ts.Statement): void;
    /**
     * Get the statements.
     */
    render(): ts.Statement[];
    /**
     * Returns an expression of all template guards that apply to this scope, including those of
     * parent scopes. If no guards have been applied, null is returned.
     */
    guards(): ts.Expression | null;
    /** Returns whether a template symbol is defined locally within the current scope. */
    isLocal(node: TmplAstVariable | TmplAstLetDeclaration | TmplAstReference): boolean;
    /**
     * Constructs a `Scope` given either a `TmplAstTemplate` or a list of `TmplAstNode`s.
     * This is identical to `Scope.forNodes` which we can't reference in some ops due to
     * circular dependencies.
     *.
     * @param parentScope the `Scope` of the parent template.
     * @param scopedNode Node that provides the scope around the child nodes (e.g. a
     * `TmplAstTemplate` node exposing variables to its children).
     * @param children Child nodes that should be appended to the TCB.
     * @param guard an expression that is applied to this scope for type narrowing purposes.
     */
    createChildScope(parentScope: Scope, scopedNode: TmplAstTemplate | TmplAstIfBlockBranch | TmplAstForLoopBlock | TmplAstHostElement | null, children: TmplAstNode[] | null, guard: ts.Expression | null): Scope;
    private resolveLocal;
    /**
     * Like `executeOp`, but assert that the operation actually returned `ts.Expression`.
     */
    private resolveOp;
    /**
     * Execute a particular `TcbOp` in the `opQueue`.
     *
     * This method replaces the operation in the `opQueue` with the result of execution (once done)
     * and also protects against a circular dependency from the operation to itself by temporarily
     * setting the operation's result to a special expression.
     */
    private executeOp;
    private appendNode;
    private appendChildren;
    private checkAndAppendReferencesOfNode;
    private appendDirectivesAndInputsOfElementLikeNode;
    private appendOutputsOfElementLikeNode;
    private appendInputsOfSelectorlessNode;
    private appendOutputsOfSelectorlessNode;
    private appendDirectiveInputs;
    private getDirectiveOp;
    private appendSelectorlessDirectives;
    private appendDeepSchemaChecks;
    private appendIcuExpressions;
    private appendContentProjectionCheckOp;
    private appendComponentNode;
    private appendDeferredBlock;
    private appendDeferredTriggers;
    private appendHostElement;
    private validateReferenceBasedDeferredTrigger;
    /** Reports a diagnostic if there are any `@let` declarations that conflict with a node. */
    private static checkConflictingLet;
}
