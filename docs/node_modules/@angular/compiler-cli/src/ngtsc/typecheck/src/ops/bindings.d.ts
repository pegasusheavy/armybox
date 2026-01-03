/*!
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import { AST, ParseSourceSpan, TmplAstBoundAttribute, TmplAstBoundEvent, TmplAstComponent, TmplAstDirective, TmplAstElement, TmplAstTemplate } from '@angular/compiler';
import ts from 'typescript';
import { TypeCheckableDirectiveMeta } from '../../api';
import { ClassPropertyName } from '../../../metadata';
import { Reference } from '../../../imports';
import { Context } from './context';
export interface TcbBoundAttribute {
    value: AST | string;
    sourceSpan: ParseSourceSpan;
    keySpan: ParseSourceSpan | null;
    inputs: {
        fieldName: ClassPropertyName;
        required: boolean;
        isSignal: boolean;
        transformType: Reference<ts.TypeNode> | null;
        isTwoWayBinding: boolean;
    }[];
}
/**
 * An input binding that corresponds with a field of a directive.
 */
export interface TcbDirectiveBoundInput {
    type: 'binding';
    /**
     * The name of a field on the directive that is set.
     */
    field: string;
    /**
     * The `ts.Expression` corresponding with the input binding expression.
     */
    expression: ts.Expression;
    /**
     * The source span of the full attribute binding.
     */
    sourceSpan: ParseSourceSpan;
    /**
     * Whether the binding is part of a two-way binding.
     */
    isTwoWayBinding: boolean;
}
/**
 * Indicates that a certain field of a directive does not have a corresponding input binding.
 */
export interface TcbDirectiveUnsetInput {
    type: 'unset';
    /**
     * The name of a field on the directive for which no input binding is present.
     */
    field: string;
}
export type TcbDirectiveInput = TcbDirectiveBoundInput | TcbDirectiveUnsetInput;
export declare function getBoundAttributes(directive: TypeCheckableDirectiveMeta, node: TmplAstTemplate | TmplAstElement | TmplAstComponent | TmplAstDirective): TcbBoundAttribute[];
export declare function checkSplitTwoWayBinding(inputName: string, output: TmplAstBoundEvent, inputs: TmplAstBoundAttribute[], tcb: Context): boolean;
/**
 * Potentially widens the type of `expr` according to the type-checking configuration.
 */
export declare function widenBinding(expr: ts.Expression, tcb: Context): ts.Expression;
