/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import ts from '../../../third_party/github.com/Microsoft/TypeScript/lib/typescript';
export declare function addVitestValueImport(imports: Set<string>, importName: string): void;
export declare function addVitestTypeImport(imports: Set<string>, importName: string): void;
export declare function getVitestAutoImports(valueImports: Set<string>, typeImports: Set<string>): ts.ImportDeclaration | undefined;
export declare function createViCallExpression(methodName: string, args?: readonly ts.Expression[], typeArgs?: ts.TypeNode[] | undefined): ts.CallExpression;
export declare function createExpectCallExpression(args: ts.Expression[], typeArgs?: ts.TypeNode[] | undefined): ts.CallExpression;
export declare function createPropertyAccess(expressionOrIndentifierText: ts.Expression | string, name: string | ts.MemberName): ts.PropertyAccessExpression;
