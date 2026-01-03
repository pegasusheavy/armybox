"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.addVitestValueImport = addVitestValueImport;
exports.addVitestTypeImport = addVitestTypeImport;
exports.getVitestAutoImports = getVitestAutoImports;
exports.createViCallExpression = createViCallExpression;
exports.createExpectCallExpression = createExpectCallExpression;
exports.createPropertyAccess = createPropertyAccess;
const typescript_1 = __importDefault(require("../../../third_party/github.com/Microsoft/TypeScript/lib/typescript"));
function addVitestValueImport(imports, importName) {
    imports.add(importName);
}
function addVitestTypeImport(imports, importName) {
    imports.add(importName);
}
function getVitestAutoImports(valueImports, typeImports) {
    if (valueImports.size === 0 && typeImports.size === 0) {
        return undefined;
    }
    const isClauseTypeOnly = valueImports.size === 0 && typeImports.size > 0;
    const allSpecifiers = [];
    // Add value imports
    for (const i of [...valueImports].sort()) {
        allSpecifiers.push(typescript_1.default.factory.createImportSpecifier(false, undefined, typescript_1.default.factory.createIdentifier(i)));
    }
    // Add type imports
    for (const i of [...typeImports].sort()) {
        // Only set isTypeOnly on individual specifiers if the clause itself is NOT type-only
        allSpecifiers.push(typescript_1.default.factory.createImportSpecifier(!isClauseTypeOnly, undefined, typescript_1.default.factory.createIdentifier(i)));
    }
    allSpecifiers.sort((a, b) => a.name.text.localeCompare(b.name.text));
    const importClause = typescript_1.default.factory.createImportClause(isClauseTypeOnly, // Set isTypeOnly on the clause if only type imports
    undefined, typescript_1.default.factory.createNamedImports(allSpecifiers));
    return typescript_1.default.factory.createImportDeclaration(undefined, importClause, typescript_1.default.factory.createStringLiteral('vitest'));
}
function createViCallExpression(methodName, args = [], typeArgs = undefined) {
    const callee = typescript_1.default.factory.createPropertyAccessExpression(typescript_1.default.factory.createIdentifier('vi'), methodName);
    return typescript_1.default.factory.createCallExpression(callee, typeArgs, args);
}
function createExpectCallExpression(args, typeArgs = undefined) {
    return typescript_1.default.factory.createCallExpression(typescript_1.default.factory.createIdentifier('expect'), typeArgs, args);
}
function createPropertyAccess(expressionOrIndentifierText, name) {
    return typescript_1.default.factory.createPropertyAccessExpression(typeof expressionOrIndentifierText === 'string'
        ? typescript_1.default.factory.createIdentifier(expressionOrIndentifierText)
        : expressionOrIndentifierText, name);
}
//# sourceMappingURL=ast-helpers.js.map