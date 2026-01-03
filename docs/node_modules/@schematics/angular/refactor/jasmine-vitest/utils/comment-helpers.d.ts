/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import ts from '../../../third_party/github.com/Microsoft/TypeScript/lib/typescript';
import { TodoCategory, TodoContextMap } from './todo-notes';
type CategoriesWithNoContext = {
    [K in TodoCategory]: TodoContextMap[K] extends never ? K : never;
}[TodoCategory];
/**
 * Adds a TODO comment to a TypeScript node for manual migration.
 * This overload handles categories that do not require a context object.
 * @param node The AST node to which the comment will be added.
 * @param category The category of the TODO, used to look up the message and URL.
 */
export declare function addTodoComment<T extends CategoriesWithNoContext>(node: ts.Node, category: T): void;
/**
 * Adds a TODO comment to a TypeScript node for manual migration.
 * This overload handles categories that require a context object, ensuring it is
 * provided and correctly typed.
 * @param node The AST node to which the comment will be added.
 * @param category The category of the TODO, used to look up the message and URL.
 * @param context The context object providing dynamic values for the message.
 */
export declare function addTodoComment<T extends TodoCategory>(node: ts.Node, category: T, context: TodoContextMap[T]): void;
export {};
