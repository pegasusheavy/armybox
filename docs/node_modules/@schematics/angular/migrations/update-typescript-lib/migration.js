"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = default_1;
const json_file_1 = require("../../utility/json-file");
const workspace_1 = require("../../utility/workspace");
function default_1() {
    return async (host, context) => {
        // Workspace level tsconfig
        if (host.exists('tsconfig.json')) {
            updateLib(host, 'tsconfig.json');
        }
        const workspace = await (0, workspace_1.getWorkspace)(host);
        // Find all tsconfig which are references used by builders
        for (const [, project] of workspace.projects) {
            for (const [targetName, target] of project.targets) {
                if (!target.options) {
                    continue;
                }
                // Update all other known CLI builders that use a tsconfig
                const tsConfigs = [target.options, ...Object.values(target.configurations || {})]
                    .filter((opt) => typeof opt?.tsConfig === 'string')
                    .map((opt) => opt.tsConfig);
                const uniqueTsConfigs = new Set(tsConfigs);
                for (const tsConfig of uniqueTsConfigs) {
                    if (host.exists(tsConfig)) {
                        updateLib(host, tsConfig);
                    }
                    else {
                        context.logger.warn(`'${tsConfig}' referenced in the '${targetName}' target does not exist.`);
                    }
                }
            }
        }
    };
}
function updateLib(host, tsConfigPath) {
    const json = new json_file_1.JSONFile(host, tsConfigPath);
    const jsonPath = ['compilerOptions', 'lib'];
    const lib = json.get(jsonPath);
    if (!lib || !Array.isArray(lib)) {
        return;
    }
    const esLibs = lib.filter((l) => typeof l === 'string' && l.toLowerCase().startsWith('es'));
    const hasDom = lib.some((l) => typeof l === 'string' && l.toLowerCase() === 'dom');
    if (esLibs.length === 0) {
        return;
    }
    const esLibToVersion = new Map();
    for (const l of esLibs) {
        const version = l.toLowerCase().match(/^es(next|(\d+))$/)?.[1];
        if (version) {
            esLibToVersion.set(l, version === 'next' ? Infinity : Number(version));
        }
    }
    if (esLibToVersion.size === 0) {
        return;
    }
    const latestEsLib = [...esLibToVersion.entries()].sort(([, v1], [, v2]) => v2 - v1)[0];
    const latestVersion = latestEsLib[1];
    if (hasDom) {
        if (latestVersion <= 2022) {
            json.remove(jsonPath);
        }
        return;
    }
    // No 'dom' with 'es' libs, so update 'es' lib.
    if (latestVersion < 2022) {
        const newLibs = lib.filter((l) => !esLibToVersion.has(l));
        newLibs.push('es2022');
        json.modify(jsonPath, newLibs);
    }
}
//# sourceMappingURL=migration.js.map