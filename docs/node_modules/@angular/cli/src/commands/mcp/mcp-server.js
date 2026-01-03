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
exports.EXPERIMENTAL_TOOLS = void 0;
exports.createMcpServer = createMcpServer;
exports.assembleToolDeclarations = assembleToolDeclarations;
const mcp_js_1 = require("@modelcontextprotocol/sdk/server/mcp.js");
const node_path_1 = __importDefault(require("node:path"));
const version_1 = require("../../utilities/version");
const instructions_1 = require("./resources/instructions");
const ai_tutor_1 = require("./tools/ai-tutor");
const best_practices_1 = require("./tools/best-practices");
const doc_search_1 = require("./tools/doc-search");
const examples_1 = require("./tools/examples");
const modernize_1 = require("./tools/modernize");
const zoneless_migration_1 = require("./tools/onpush-zoneless-migration/zoneless-migration");
const projects_1 = require("./tools/projects");
const tool_registry_1 = require("./tools/tool-registry");
/**
 * The set of tools that are enabled by default for the MCP server.
 * These tools are considered stable and suitable for general use.
 */
const STABLE_TOOLS = [
    ai_tutor_1.AI_TUTOR_TOOL,
    best_practices_1.BEST_PRACTICES_TOOL,
    doc_search_1.DOC_SEARCH_TOOL,
    examples_1.FIND_EXAMPLE_TOOL,
    projects_1.LIST_PROJECTS_TOOL,
    zoneless_migration_1.ZONELESS_MIGRATION_TOOL,
];
/**
 * The set of tools that are available but not enabled by default.
 * These tools are considered experimental and may have limitations.
 */
exports.EXPERIMENTAL_TOOLS = [modernize_1.MODERNIZE_TOOL];
async function createMcpServer(options, logger) {
    const server = new mcp_js_1.McpServer({
        name: 'angular-cli-server',
        version: version_1.VERSION.full,
    }, {
        capabilities: {
            resources: {},
            tools: {},
            logging: {},
        },
        instructions: `
<General Purpose>
This server provides a safe, programmatic interface to the Angular CLI for an AI assistant.
Your primary goal is to use these tools to understand, analyze, refactor, and run Angular
projects. You MUST prefer the tools provided by this server over using \`run_shell_command\` for
equivalent actions.
</General Purpose>

<Core Workflows & Tool Guide>
* **1. Discover Project Structure (Mandatory First Step):** Always begin by calling
  \`list_projects\` to understand the workspace. The \`path\` property for a workspace
  is a required input for other tools.

* **2. Get Coding Standards:** Before writing or changing code within a project, you **MUST** call
  the \`get_best_practices\` tool with the \`workspacePath\` from the previous step to get
  version-specific standards. For general knowledge, you can call the tool without this path.

* **3. Answer User Questions:**
    - For conceptual questions ("what is..."), use \`search_documentation\`.
    - For code examples ("show me how to..."), use \`find_examples\`.
</Core Workflows & Tool Guide>

<Key Concepts>
* **Workspace vs. Project:** A 'workspace' contains an \`angular.json\` file and defines 'projects'
  (applications or libraries). A monorepo can have multiple workspaces.
* **Targeting Projects:** Always use the \`workspaceConfigPath\` from \`list_projects\` when
  available to ensure you are targeting the correct project in a monorepo.
</Key Concepts>
`,
    });
    (0, instructions_1.registerInstructionsResource)(server);
    const toolDeclarations = assembleToolDeclarations(STABLE_TOOLS, exports.EXPERIMENTAL_TOOLS, {
        ...options,
        logger,
    });
    await (0, tool_registry_1.registerTools)(server, {
        workspace: options.workspace,
        logger,
        exampleDatabasePath: node_path_1.default.join(__dirname, '../../../lib/code-examples.db'),
    }, toolDeclarations);
    return server;
}
function assembleToolDeclarations(stableDeclarations, experimentalDeclarations, options) {
    let toolDeclarations = [...stableDeclarations];
    if (options.readOnly) {
        toolDeclarations = toolDeclarations.filter((tool) => tool.isReadOnly);
    }
    if (options.localOnly) {
        toolDeclarations = toolDeclarations.filter((tool) => tool.isLocalOnly);
    }
    const enabledExperimentalTools = new Set(options.experimentalTools);
    if (process.env['NG_MCP_CODE_EXAMPLES'] === '1') {
        enabledExperimentalTools.add('find_examples');
    }
    if (enabledExperimentalTools.size > 0) {
        const experimentalToolsMap = new Map(experimentalDeclarations.map((tool) => [tool.name, tool]));
        for (const toolName of enabledExperimentalTools) {
            const tool = experimentalToolsMap.get(toolName);
            if (tool) {
                toolDeclarations.push(tool);
            }
            else {
                options.logger.warn(`Unknown experimental tool: ${toolName}`);
            }
        }
    }
    return toolDeclarations;
}
//# sourceMappingURL=mcp-server.js.map