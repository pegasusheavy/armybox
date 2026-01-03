/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { DirectiveOwner, TmplAstComponent, TmplAstDirective, TmplAstElement, TmplAstNode, TmplAstTemplate } from '@angular/compiler';
import { TcbOp } from './base';
import type { Context } from './context';
import type { Scope } from './scope';
import { TypeCheckableDirectiveMeta } from '../../api';
import { TcbBoundAttribute } from './bindings';
/** Possible types of custom field directives. */
export type CustomFieldType = 'value' | 'checkbox';
/** Names of input fields to which users aren't allowed to bind when using a `field` directive. */
export declare const customFormControlBannedInputFields: Set<string>;
/**
 * A `TcbOp` which constructs an instance of the signal forms `Field` directive on a native element.
 */
export declare class TcbNativeFieldOp extends TcbOp {
    protected tcb: Context;
    protected scope: Scope;
    protected node: TmplAstElement;
    private inputType;
    /** Bindings that aren't supported on signal form fields. */
    protected readonly unsupportedBindingFields: Set<string>;
    get optional(): boolean;
    constructor(tcb: Context, scope: Scope, node: TmplAstElement, inputType: string | null);
    execute(): null;
    private getExpectedTypeFromDomNode;
    private getUnsupportedType;
}
/**
 * A variation of the `TcbNativeFieldOp` with specific logic for radio buttons.
 */
export declare class TcbNativeRadioButtonFieldOp extends TcbNativeFieldOp {
    constructor(tcb: Context, scope: Scope, node: TmplAstElement);
    execute(): null;
}
/** Expands the set of bound inputs with the ones from custom field directives. */
export declare function expandBoundAttributesForField(directive: TypeCheckableDirectiveMeta, node: TmplAstTemplate | TmplAstElement | TmplAstComponent | TmplAstDirective, customFieldType: CustomFieldType): TcbBoundAttribute[] | null;
export declare function isFieldDirective(meta: TypeCheckableDirectiveMeta): boolean;
/** Determines if a directive is a custom field and its type. */
export declare function getCustomFieldDirectiveType(meta: TypeCheckableDirectiveMeta): CustomFieldType | null;
/** Determines if a directive usage is on a native field. */
export declare function isNativeField(dir: TypeCheckableDirectiveMeta, node: TmplAstNode, allDirectiveMatches: TypeCheckableDirectiveMeta[]): node is TmplAstElement & {
    name: 'input' | 'select' | 'textarea';
};
/** Checks whether a node has bindings that aren't supported on fields. */
export declare function checkUnsupportedFieldBindings(node: DirectiveOwner, unsupportedBindingFields: Set<string>, tcb: Context): void;
