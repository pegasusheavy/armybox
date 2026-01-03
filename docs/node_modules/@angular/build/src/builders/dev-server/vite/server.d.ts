/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import type { Connect, InlineConfig } from 'vite';
import type { ComponentStyleRecord } from '../../../tools/vite/middlewares';
import { ServerSsrMode } from '../../../tools/vite/plugins';
import { EsbuildLoaderOption } from '../../../tools/vite/utils';
import { type ApplicationBuilderInternalOptions, JavaScriptTransformer } from '../internal';
import type { NormalizedDevServerOptions } from '../options';
import { DevServerExternalResultMetadata, OutputAssetRecord, OutputFileRecord } from './utils';
export declare function setupServer(serverOptions: NormalizedDevServerOptions, outputFiles: Map<string, OutputFileRecord>, assets: Map<string, OutputAssetRecord>, preserveSymlinks: boolean | undefined, externalMetadata: DevServerExternalResultMetadata, ssrMode: ServerSsrMode, prebundleTransformer: JavaScriptTransformer, target: string[], zoneless: boolean, componentStyles: Map<string, ComponentStyleRecord>, templateUpdates: Map<string, string>, prebundleLoaderExtensions: EsbuildLoaderOption | undefined, define: ApplicationBuilderInternalOptions['define'], extensionMiddleware?: Connect.NextHandleFunction[], indexHtmlTransformer?: (content: string) => Promise<string>, thirdPartySourcemaps?: boolean): Promise<InlineConfig>;
