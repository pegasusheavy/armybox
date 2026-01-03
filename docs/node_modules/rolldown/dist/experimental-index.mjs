import { n as __toESM, t as require_binding } from "./shared/binding-DBWhurrz.mjs";
import { b as BuiltinPlugin, c as createBundlerOptions, f as normalizeBindingResult, l as normalizedStringOrRegex, m as bindingifySourcemap, p as transformToRollupOutput, s as RolldownBuild, x as makeBuiltinPluginCallable, y as PluginDriver } from "./shared/src-BwxUhqZU.mjs";
import "./shared/parse-ast-index-B4DJl5-M.mjs";
import "./shared/misc-usdOVIou.mjs";
import { pathToFileURL } from "node:url";

//#region src/api/dev/dev-engine.ts
var import_binding$1 = /* @__PURE__ */ __toESM(require_binding(), 1);
var DevEngine = class DevEngine {
	#inner;
	#cachedBuildFinishPromise = null;
	static async create(inputOptions, outputOptions = {}, devOptions = {}) {
		inputOptions = await PluginDriver.callOptionsHook(inputOptions);
		const options = await createBundlerOptions(inputOptions, outputOptions, false);
		const userOnHmrUpdates = devOptions.onHmrUpdates;
		const bindingOnHmrUpdates = userOnHmrUpdates ? function(rawResult) {
			const result = normalizeBindingResult(rawResult);
			if (result instanceof Error) {
				userOnHmrUpdates(result);
				return;
			}
			const [updates, changedFiles] = result;
			userOnHmrUpdates({
				updates,
				changedFiles
			});
		} : void 0;
		const userOnOutput = devOptions.onOutput;
		const bindingDevOptions = {
			onHmrUpdates: bindingOnHmrUpdates,
			onOutput: userOnOutput ? function(rawResult) {
				const result = normalizeBindingResult(rawResult);
				if (result instanceof Error) {
					userOnOutput(result);
					return;
				}
				userOnOutput(transformToRollupOutput(result));
			} : void 0,
			rebuildStrategy: devOptions.rebuildStrategy ? devOptions.rebuildStrategy === "always" ? import_binding$1.BindingRebuildStrategy.Always : devOptions.rebuildStrategy === "auto" ? import_binding$1.BindingRebuildStrategy.Auto : import_binding$1.BindingRebuildStrategy.Never : void 0,
			watch: devOptions.watch && {
				skipWrite: devOptions.watch.skipWrite,
				usePolling: devOptions.watch.usePolling,
				pollInterval: devOptions.watch.pollInterval,
				useDebounce: devOptions.watch.useDebounce,
				debounceDuration: devOptions.watch.debounceDuration,
				compareContentsForPolling: devOptions.watch.compareContentsForPolling,
				debounceTickRate: devOptions.watch.debounceTickRate
			}
		};
		return new DevEngine(new import_binding$1.BindingDevEngine(options.bundlerOptions, bindingDevOptions));
	}
	constructor(inner) {
		this.#inner = inner;
	}
	async run() {
		await this.#inner.run();
	}
	async ensureCurrentBuildFinish() {
		if (this.#cachedBuildFinishPromise) return this.#cachedBuildFinishPromise;
		const promise = this.#inner.ensureCurrentBuildFinish().then(() => {
			this.#cachedBuildFinishPromise = null;
		});
		this.#cachedBuildFinishPromise = promise;
		return promise;
	}
	async hasLatestBuildOutput() {
		return this.#inner.hasLatestBuildOutput();
	}
	async ensureLatestBuildOutput() {
		await this.#inner.ensureLatestBuildOutput();
	}
	async invalidate(file, firstInvalidatedBy) {
		return this.#inner.invalidate(file, firstInvalidatedBy);
	}
	registerModules(clientId, modules) {
		this.#inner.registerModules(clientId, modules);
	}
	removeClient(clientId) {
		this.#inner.removeClient(clientId);
	}
	async close() {
		await this.#inner.close();
	}
};

//#endregion
//#region src/api/dev/index.ts
const dev = (...args) => DevEngine.create(...args);

//#endregion
//#region src/types/external-memory-handle.ts
const symbolForExternalMemoryHandle = "__rolldown_external_memory_handle__";
/**
* Frees the external memory held by the given handle.
*
* This is useful when you want to manually release memory held by Rust objects
* (like `OutputChunk` or `OutputAsset`) before they are garbage collected.
*
* @param handle - The object with external memory to free
* @param keepDataAlive - If true, evaluates all lazy fields before freeing memory (default: false).
*   This will take time to copy data from Rust to JavaScript, but prevents errors
*   when accessing properties after the memory is freed.
* @returns Status object with `freed` boolean and optional `reason` string.
*   - `{ freed: true }` if memory was successfully freed
*   - `{ freed: false, reason: "..." }` if memory couldn't be freed (e.g., already freed or other references exist)
*
* @example
* ```typescript
* import { freeExternalMemory } from 'rolldown/experimental';
*
* const output = await bundle.generate();
* const chunk = output.output[0];
*
* // Use the chunk...
*
* // Manually free the memory (fast, but accessing properties after will throw)
* const status = freeExternalMemory(chunk); // { freed: true }
* const statusAgain = freeExternalMemory(chunk); // { freed: false, reason: "Memory has already been freed" }
*
* // Keep data alive before freeing (slower, but data remains accessible)
* freeExternalMemory(chunk, true); // Evaluates all lazy fields first
* console.log(chunk.code); // OK - data was copied to JavaScript before freeing
*
* // Without keepDataAlive, accessing chunk properties after freeing will throw an error
* ```
*/
function freeExternalMemory(handle, keepDataAlive = false) {
	return handle[symbolForExternalMemoryHandle](keepDataAlive);
}

//#endregion
//#region src/api/experimental.ts
/**
* This is an experimental API. It's behavior may change in the future.
*
* Calling this API will only execute the scan stage of rolldown.
*/
const scan = async (input) => {
	const build = new RolldownBuild(await PluginDriver.callOptionsHook(input));
	try {
		await build.scan();
	} finally {
		await build.close();
	}
};

//#endregion
//#region src/plugin/parallel-plugin.ts
function defineParallelPlugin(pluginPath) {
	return (options) => {
		return { _parallel: {
			fileUrl: pathToFileURL(pluginPath).href,
			options
		} };
	};
}

//#endregion
//#region src/builtin-plugin/constructors.ts
function modulePreloadPolyfillPlugin(config) {
	return new BuiltinPlugin("builtin:module-preload-polyfill", config);
}
function dynamicImportVarsPlugin(config) {
	if (config) {
		config.include = normalizedStringOrRegex(config.include);
		config.exclude = normalizedStringOrRegex(config.exclude);
	}
	return new BuiltinPlugin("builtin:dynamic-import-vars", config);
}
function importGlobPlugin(config) {
	return new BuiltinPlugin("builtin:import-glob", config);
}
function reporterPlugin(config) {
	return new BuiltinPlugin("builtin:reporter", config);
}
function manifestPlugin(config) {
	return new BuiltinPlugin("builtin:manifest", config);
}
function wasmHelperPlugin(config) {
	return new BuiltinPlugin("builtin:wasm-helper", config);
}
function wasmFallbackPlugin() {
	return makeBuiltinPluginCallable(new BuiltinPlugin("builtin:wasm-fallback"));
}
function loadFallbackPlugin() {
	return new BuiltinPlugin("builtin:load-fallback");
}
function jsonPlugin(config) {
	return makeBuiltinPluginCallable(new BuiltinPlugin("builtin:json", config));
}
function buildImportAnalysisPlugin(config) {
	return new BuiltinPlugin("builtin:build-import-analysis", config);
}
function viteResolvePlugin(config) {
	return makeBuiltinPluginCallable(new BuiltinPlugin("builtin:vite-resolve", config));
}
function isolatedDeclarationPlugin(config) {
	return new BuiltinPlugin("builtin:isolated-declaration", config);
}
function webWorkerPostPlugin() {
	return new BuiltinPlugin("builtin:web-worker-post");
}
function esmExternalRequirePlugin(config) {
	return new BuiltinPlugin("builtin:esm-external-require", config);
}
function reactRefreshWrapperPlugin(config) {
	if (config) {
		config.include = normalizedStringOrRegex(config.include);
		config.exclude = normalizedStringOrRegex(config.exclude);
	}
	return makeBuiltinPluginCallable(new BuiltinPlugin("builtin:react-refresh-wrapper", config));
}
function viteCSSPostPlugin(config) {
	return new BuiltinPlugin("builtin:vite-css-post", config);
}
function viteHtmlPlugin(config) {
	return new BuiltinPlugin("builtin:vite-html", config);
}

//#endregion
//#region src/builtin-plugin/alias-plugin.ts
function aliasPlugin(config) {
	return new BuiltinPlugin("builtin:alias", config);
}

//#endregion
//#region src/builtin-plugin/asset-plugin.ts
function assetPlugin(config) {
	return new BuiltinPlugin("builtin:asset", config);
}

//#endregion
//#region src/builtin-plugin/replace-plugin.ts
/**
* Replaces targeted strings in files while bundling.
*
* @example
* // Basic usage
* ```js
* replacePlugin({
*   'process.env.NODE_ENV': JSON.stringify('production'),
*    __buildVersion: 15
* })
* ```
* @example
* // With options
* ```js
* replacePlugin({
*   'process.env.NODE_ENV': JSON.stringify('production'),
*   __buildVersion: 15
* }, {
*   preventAssignment: false,
* })
* ```
*/
function replacePlugin(values = {}, options = {}) {
	Object.keys(values).forEach((key) => {
		const value = values[key];
		if (typeof value !== "string") values[key] = String(value);
	});
	return makeBuiltinPluginCallable(new BuiltinPlugin("builtin:replace", {
		...options,
		values
	}));
}

//#endregion
//#region src/builtin-plugin/transform-plugin.ts
function transformPlugin(config) {
	if (config) config = {
		...config,
		include: normalizedStringOrRegex(config.include),
		exclude: normalizedStringOrRegex(config.exclude),
		jsxRefreshInclude: normalizedStringOrRegex(config.jsxRefreshInclude),
		jsxRefreshExclude: normalizedStringOrRegex(config.jsxRefreshExclude)
	};
	return new BuiltinPlugin("builtin:transform", config);
}

//#endregion
//#region src/builtin-plugin/vite-css-plugin.ts
function viteCSSPlugin(config) {
	return new BuiltinPlugin("builtin:vite-css", config ? {
		...config,
		async compileCSS(url, importer, resolver) {
			let result = await config.compileCSS(url, importer, resolver);
			return {
				...result,
				map: bindingifySourcemap(result.map)
			};
		}
	} : void 0);
}

//#endregion
//#region src/experimental-index.ts
var import_binding = /* @__PURE__ */ __toESM(require_binding(), 1);

//#endregion
var BindingRebuildStrategy = import_binding.BindingRebuildStrategy;
var ResolverFactory = import_binding.ResolverFactory;
var isolatedDeclaration = import_binding.isolatedDeclaration;
var minify = import_binding.minify;
var moduleRunnerTransform = import_binding.moduleRunnerTransform;
var transform = import_binding.transform;
export { BindingRebuildStrategy, DevEngine, ResolverFactory, aliasPlugin, assetPlugin, buildImportAnalysisPlugin, defineParallelPlugin, dev, dynamicImportVarsPlugin, esmExternalRequirePlugin, freeExternalMemory, importGlobPlugin, isolatedDeclaration, isolatedDeclarationPlugin, jsonPlugin, loadFallbackPlugin, manifestPlugin, minify, modulePreloadPolyfillPlugin, moduleRunnerTransform, reactRefreshWrapperPlugin, replacePlugin, reporterPlugin, scan, transform, transformPlugin, viteCSSPlugin, viteCSSPostPlugin, viteHtmlPlugin, viteResolvePlugin, wasmFallbackPlugin, wasmHelperPlugin, webWorkerPostPlugin };