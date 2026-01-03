import { n as __toESM, t as require_binding } from "./binding-DBWhurrz.mjs";
import { styleText } from "node:util";

//#region src/utils/code-frame.ts
function spaces(index) {
	let result = "";
	while (index--) result += " ";
	return result;
}
function tabsToSpaces(value) {
	return value.replace(/^\t+/, (match) => match.split("	").join("  "));
}
const LINE_TRUNCATE_LENGTH = 120;
const MIN_CHARACTERS_SHOWN_AFTER_LOCATION = 10;
const ELLIPSIS = "...";
function getCodeFrame(source, line, column) {
	let lines = source.split("\n");
	if (line > lines.length) return "";
	const maxLineLength = Math.max(tabsToSpaces(lines[line - 1].slice(0, column)).length + MIN_CHARACTERS_SHOWN_AFTER_LOCATION + 3, LINE_TRUNCATE_LENGTH);
	const frameStart = Math.max(0, line - 3);
	let frameEnd = Math.min(line + 2, lines.length);
	lines = lines.slice(frameStart, frameEnd);
	while (!/\S/.test(lines[lines.length - 1])) {
		lines.pop();
		frameEnd -= 1;
	}
	const digits = String(frameEnd).length;
	return lines.map((sourceLine, index) => {
		const isErrorLine = frameStart + index + 1 === line;
		let lineNumber = String(index + frameStart + 1);
		while (lineNumber.length < digits) lineNumber = ` ${lineNumber}`;
		let displayedLine = tabsToSpaces(sourceLine);
		if (displayedLine.length > maxLineLength) displayedLine = `${displayedLine.slice(0, maxLineLength - 3)}${ELLIPSIS}`;
		if (isErrorLine) {
			const indicator = spaces(digits + 2 + tabsToSpaces(sourceLine.slice(0, column)).length) + "^";
			return `${lineNumber}: ${displayedLine}\n${indicator}`;
		}
		return `${lineNumber}: ${displayedLine}`;
	}).join("\n");
}

//#endregion
//#region src/utils/style-text.ts
/**
* Cross-platform styleText utility that works in both Node.js and browser environments
* In Node.js, it uses the native `styleText` from `node:util`
* In browser, it provides empty styling functions for compatibility
*/
function styleText$1(...args) {
	return styleText(...args);
}

//#endregion
//#region src/log/locate-character/index.js
/** @typedef {import('./types').Location} Location */
/**
* @param {import('./types').Range} range
* @param {number} index
*/
function rangeContains(range, index) {
	return range.start <= index && index < range.end;
}
/**
* @param {string} source
* @param {import('./types').Options} [options]
*/
function getLocator(source, options = {}) {
	const { offsetLine = 0, offsetColumn = 0 } = options;
	let start = 0;
	const ranges = source.split("\n").map((line, i$1) => {
		const end = start + line.length + 1;
		/** @type {import('./types').Range} */
		const range = {
			start,
			end,
			line: i$1
		};
		start = end;
		return range;
	});
	let i = 0;
	/**
	* @param {string | number} search
	* @param {number} [index]
	* @returns {Location | undefined}
	*/
	function locator(search, index) {
		if (typeof search === "string") search = source.indexOf(search, index ?? 0);
		if (search === -1) return void 0;
		let range = ranges[i];
		const d = search >= range.end ? 1 : -1;
		while (range) {
			if (rangeContains(range, search)) return {
				line: offsetLine + range.line,
				column: offsetColumn + search - range.start,
				character: search
			};
			i += d;
			range = ranges[i];
		}
	}
	return locator;
}
/**
* @param {string} source
* @param {string | number} search
* @param {import('./types').Options} [options]
* @returns {Location | undefined}
*/
function locate(source, search, options) {
	return getLocator(source, options)(search, options && options.startIndex);
}

//#endregion
//#region src/log/logs.ts
const INVALID_LOG_POSITION = "INVALID_LOG_POSITION", PLUGIN_ERROR = "PLUGIN_ERROR", INPUT_HOOK_IN_OUTPUT_PLUGIN = "INPUT_HOOK_IN_OUTPUT_PLUGIN", CYCLE_LOADING = "CYCLE_LOADING", MULTIPLY_NOTIFY_OPTION = "MULTIPLY_NOTIFY_OPTION", PARSE_ERROR = "PARSE_ERROR", NO_FS_IN_BROWSER = "NO_FS_IN_BROWSER", DEPRECATED_DEFINE = "DEPRECATED_DEFINE", DEPRECATED_INJECT = "DEPRECATED_INJECT", DEPRECATED_PROFILER_NAMES = "DEPRECATED_PROFILER_NAMES", DEPRECATED_KEEP_NAMES = "DEPRECATED_KEEP_NAMES", DEPRECATED_DROP_LABELS = "DEPRECATED_DROP_LABELS";
function logParseError(message) {
	return {
		code: PARSE_ERROR,
		message
	};
}
function logInvalidLogPosition(pluginName) {
	return {
		code: INVALID_LOG_POSITION,
		message: `Plugin "${pluginName}" tried to add a file position to a log or warning. This is only supported in the "transform" hook at the moment and will be ignored.`
	};
}
function logInputHookInOutputPlugin(pluginName, hookName) {
	return {
		code: INPUT_HOOK_IN_OUTPUT_PLUGIN,
		message: `The "${hookName}" hook used by the output plugin ${pluginName} is a build time hook and will not be run for that plugin. Either this plugin cannot be used as an output plugin, or it should have an option to configure it as an output plugin.`
	};
}
function logCycleLoading(pluginName, moduleId) {
	return {
		code: CYCLE_LOADING,
		message: `Found the module "${moduleId}" cycle loading at ${pluginName} plugin, it maybe blocking fetching modules.`
	};
}
function logMultiplyNotifyOption() {
	return {
		code: MULTIPLY_NOTIFY_OPTION,
		message: `Found multiply notify option at watch options, using first one to start notify watcher.`
	};
}
function logDeprecatedDefine() {
	return {
		code: DEPRECATED_DEFINE,
		message: `${styleText$1(["yellow", "bold"], "⚠ Deprecation Warning:")} The top-level "define" option is deprecated. Use "transform.define" instead.`
	};
}
function logDeprecatedInject() {
	return {
		code: DEPRECATED_INJECT,
		message: `${styleText$1(["yellow", "bold"], "⚠ Deprecation Warning:")} The top-level "inject" option is deprecated. Use "transform.inject" instead.`
	};
}
function logDeprecatedProfilerNames() {
	return {
		code: DEPRECATED_PROFILER_NAMES,
		message: "The top-level \"profilerNames\" option is deprecated. Use \"output.generatedCode.profilerNames\" instead."
	};
}
function logDeprecatedKeepNames() {
	return {
		code: DEPRECATED_KEEP_NAMES,
		message: "The top-level \"keepNames\" option is deprecated. Use \"output.keepNames\" instead."
	};
}
function logDeprecatedDropLabels() {
	return {
		code: DEPRECATED_DROP_LABELS,
		message: `${styleText$1(["yellow", "bold"], "⚠ Deprecation Warning:")} The top-level "dropLabels" option is deprecated. Use "transform.dropLabels" instead.`
	};
}
function logPluginError(error$1, plugin, { hook, id } = {}) {
	try {
		const code = error$1.code;
		if (!error$1.pluginCode && code != null && (typeof code !== "string" || !code.startsWith("PLUGIN_"))) error$1.pluginCode = code;
		error$1.code = PLUGIN_ERROR;
		error$1.plugin = plugin;
		if (hook) error$1.hook = hook;
		if (id) error$1.id = id;
	} catch (_) {} finally {
		return error$1;
	}
}
function error(base) {
	if (!(base instanceof Error)) {
		base = Object.assign(new Error(base.message), base);
		Object.defineProperty(base, "name", {
			value: "RollupError",
			writable: true
		});
	}
	throw base;
}
function augmentCodeLocation(properties, pos, source, id) {
	if (typeof pos === "object") {
		const { line, column } = pos;
		properties.loc = {
			column,
			file: id,
			line
		};
	} else {
		properties.pos = pos;
		const location = locate(source, pos, { offsetLine: 1 });
		if (!location) return;
		const { line, column } = location;
		properties.loc = {
			column,
			file: id,
			line
		};
	}
	if (properties.frame === void 0) {
		const { line, column } = properties.loc;
		properties.frame = getCodeFrame(source, line, column);
	}
}

//#endregion
//#region ../../node_modules/.pnpm/oxc-parser@0.96.0/node_modules/oxc-parser/src-js/wrap.js
function wrap$1(result) {
	let program, module, comments, errors;
	return {
		get program() {
			if (!program) program = jsonParseAst(result.program);
			return program;
		},
		get module() {
			if (!module) module = result.module;
			return module;
		},
		get comments() {
			if (!comments) comments = result.comments;
			return comments;
		},
		get errors() {
			if (!errors) errors = result.errors;
			return errors;
		}
	};
}
function jsonParseAst(programJson) {
	const { node: program, fixes } = JSON.parse(programJson);
	for (const fixPath of fixes) applyFix(program, fixPath);
	return program;
}
function applyFix(program, fixPath) {
	let node = program;
	for (const key of fixPath) node = node[key];
	if (node.bigint) node.value = BigInt(node.bigint);
	else try {
		node.value = RegExp(node.regex.pattern, node.regex.flags);
	} catch {}
}

//#endregion
//#region src/parse-ast-index.ts
var import_binding = /* @__PURE__ */ __toESM(require_binding(), 1);
function wrap(result, sourceText) {
	result = wrap$1(result);
	if (result.errors.length > 0) return normalizeParseError(sourceText, result.errors);
	return result.program;
}
function normalizeParseError(sourceText, errors) {
	let message = `Parse failed with ${errors.length} error${errors.length < 2 ? "" : "s"}:\n`;
	for (let i = 0; i < errors.length; i++) {
		if (i >= 5) {
			message += "\n...";
			break;
		}
		const e = errors[i];
		message += e.message + "\n" + e.labels.map((label) => {
			const location = locate(sourceText, label.start, { offsetLine: 1 });
			if (!location) return;
			return getCodeFrame(sourceText, location.line, location.column);
		}).filter(Boolean).join("\n");
	}
	return error(logParseError(message));
}
const defaultParserOptions = {
	lang: "js",
	preserveParens: false
};
function parseAst(sourceText, options, filename) {
	return wrap((0, import_binding.parseSync)(filename ?? "file.js", sourceText, {
		...defaultParserOptions,
		...options
	}), sourceText);
}
async function parseAstAsync(sourceText, options, filename) {
	return wrap(await (0, import_binding.parseAsync)(filename ?? "file.js", sourceText, {
		...defaultParserOptions,
		...options
	}), sourceText);
}

//#endregion
export { logCycleLoading as a, logDeprecatedInject as c, logInputHookInOutputPlugin as d, logInvalidLogPosition as f, styleText$1 as h, error as i, logDeprecatedKeepNames as l, logPluginError as m, parseAstAsync as n, logDeprecatedDefine as o, logMultiplyNotifyOption as p, augmentCodeLocation as r, logDeprecatedDropLabels as s, parseAst as t, logDeprecatedProfilerNames as u };