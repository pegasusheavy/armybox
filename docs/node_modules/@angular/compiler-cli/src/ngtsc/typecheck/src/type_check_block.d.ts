/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import ts from 'typescript';
import { Reference } from '../../imports';
import { ClassDeclaration } from '../../reflection';
import { TypeCheckBlockMetadata } from '../api';
import { DomSchemaChecker } from './dom';
import { Environment } from './environment';
import { OutOfBandDiagnosticRecorder } from './oob';
import { TcbGenericContextBehavior } from './ops/context';
/**
 * Given a `ts.ClassDeclaration` for a component, and metadata regarding that component, compose a
 * "type check block" function.
 *
 * When passed through TypeScript's TypeChecker, type errors that arise within the type check block
 * function indicate issues in the template itself.
 *
 * As a side effect of generating a TCB for the component, `ts.Diagnostic`s may also be produced
 * directly for issues within the template which are identified during generation. These issues are
 * recorded in either the `domSchemaChecker` (which checks usage of DOM elements and bindings) as
 * well as the `oobRecorder` (which records errors when the type-checking code generator is unable
 * to sufficiently understand a template).
 *
 * @param env an `Environment` into which type-checking code will be generated.
 * @param ref a `Reference` to the component class which should be type-checked.
 * @param name a `ts.Identifier` to use for the generated `ts.FunctionDeclaration`.
 * @param meta metadata about the component's template and the function being generated.
 * @param domSchemaChecker used to check and record errors regarding improper usage of DOM elements
 * and bindings.
 * @param oobRecorder used to record errors regarding template elements which could not be correctly
 * translated into types during TCB generation.
 * @param genericContextBehavior controls how generic parameters (especially parameters with generic
 * bounds) will be referenced from the generated TCB code.
 */
export declare function generateTypeCheckBlock(env: Environment, ref: Reference<ClassDeclaration<ts.ClassDeclaration>>, name: ts.Identifier, meta: TypeCheckBlockMetadata, domSchemaChecker: DomSchemaChecker, oobRecorder: OutOfBandDiagnosticRecorder, genericContextBehavior: TcbGenericContextBehavior): ts.FunctionDeclaration;
