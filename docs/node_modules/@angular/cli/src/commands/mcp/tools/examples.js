"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.FIND_EXAMPLE_TOOL = void 0;
exports.escapeSearchQuery = escapeSearchQuery;
const promises_1 = require("node:fs/promises");
const node_module_1 = require("node:module");
const node_path_1 = __importDefault(require("node:path"));
const zod_1 = require("zod");
const tool_registry_1 = require("./tool-registry");
const findExampleInputSchema = zod_1.z.object({
    workspacePath: zod_1.z
        .string()
        .optional()
        .describe('The absolute path to the `angular.json` file for the workspace. This is used to find the ' +
        'version-specific code examples that correspond to the installed version of the ' +
        'Angular framework. You **MUST** get this path from the `list_projects` tool. ' +
        'If omitted, the tool will search the generic code examples bundled with the CLI.'),
    query: zod_1.z
        .string()
        .describe(`The primary, conceptual search query. This should capture the user's main goal or question ` +
        `(e.g., 'lazy loading a route' or 'how to use signal inputs'). The query will be processed ` +
        'by a powerful full-text search engine.\n\n' +
        'Key Syntax Features (see https://www.sqlite.org/fts5.html for full documentation):\n' +
        '  - AND (default): Space-separated terms are combined with AND.\n' +
        '    - Example: \'standalone component\' (finds results with both "standalone" and "component")\n' +
        '  - OR: Use the OR operator to find results with either term.\n' +
        "    - Example: 'validation OR validator'\n" +
        '  - NOT: Use the NOT operator to exclude terms.\n' +
        "    - Example: 'forms NOT reactive'\n" +
        '  - Grouping: Use parentheses () to group expressions.\n' +
        "    - Example: '(validation OR validator) AND forms'\n" +
        '  - Phrase Search: Use double quotes "" for exact phrases.\n' +
        '    - Example: \'"template-driven forms"\'\n' +
        '  - Prefix Search: Use an asterisk * for prefix matching.\n' +
        '    - Example: \'rout*\' (matches "route", "router", "routing")'),
    keywords: zod_1.z
        .array(zod_1.z.string())
        .optional()
        .describe('A list of specific, exact keywords to narrow the search. Use this for precise terms like ' +
        'API names, function names, or decorators (e.g., `ngFor`, `trackBy`, `inject`).'),
    required_packages: zod_1.z
        .array(zod_1.z.string())
        .optional()
        .describe("A list of NPM packages that an example must use. Use this when the user's request is " +
        'specific to a feature within a certain package (e.g., if the user asks about `ngModel`, ' +
        'you should filter by `@angular/forms`).'),
    related_concepts: zod_1.z
        .array(zod_1.z.string())
        .optional()
        .describe('A list of high-level concepts to filter by. Use this to find examples related to broader ' +
        'architectural ideas or patterns (e.g., `signals`, `dependency injection`, `routing`).'),
    includeExperimental: zod_1.z
        .boolean()
        .optional()
        .default(false)
        .describe('By default, this tool returns only production-safe examples. Set this to `true` **only if** ' +
        'the user explicitly asks for a bleeding-edge feature or if a stable solution to their ' +
        'problem cannot be found. If you set this to `true`, you **MUST** preface your answer by ' +
        'warning the user that the example uses experimental APIs that are not suitable for production.'),
});
const findExampleOutputSchema = zod_1.z.object({
    examples: zod_1.z.array(zod_1.z.object({
        title: zod_1.z
            .string()
            .describe('The title of the example. Use this as a heading when presenting the example to the user.'),
        summary: zod_1.z
            .string()
            .describe("A one-sentence summary of the example's purpose. Use this to help the user decide " +
            'if the example is relevant to them.'),
        keywords: zod_1.z
            .array(zod_1.z.string())
            .optional()
            .describe('A list of keywords for the example. You can use these to explain why this example ' +
            "was a good match for the user's query."),
        required_packages: zod_1.z
            .array(zod_1.z.string())
            .optional()
            .describe('A list of NPM packages required for the example to work. Before presenting the code, ' +
            'you should inform the user if any of these packages need to be installed.'),
        related_concepts: zod_1.z
            .array(zod_1.z.string())
            .optional()
            .describe('A list of related concepts. You can suggest these to the user as topics for ' +
            'follow-up questions.'),
        related_tools: zod_1.z
            .array(zod_1.z.string())
            .optional()
            .describe('A list of related MCP tools. You can suggest these as potential next steps for the user.'),
        content: zod_1.z
            .string()
            .describe('A complete, self-contained Angular code example in Markdown format. This should be ' +
            'presented to the user inside a markdown code block.'),
        snippet: zod_1.z
            .string()
            .optional()
            .describe('A contextual snippet from the content showing the matched search term. This field is ' +
            'critical for efficiently evaluating a result`s relevance. It enables two primary ' +
            'workflows:\n\n' +
            '1. For direct questions: You can internally review snippets to select the single best ' +
            'result before generating a comprehensive answer from its full `content`.\n' +
            '2. For ambiguous or exploratory questions: You can present a summary of titles and ' +
            'snippets to the user, allowing them to guide the next step.'),
    })),
});
exports.FIND_EXAMPLE_TOOL = (0, tool_registry_1.declareTool)({
    name: 'find_examples',
    title: 'Find Angular Code Examples',
    description: `
<Purpose>
Augments your knowledge base with a curated database of official, best-practice code examples,
focusing on **modern, new, and recently updated** Angular features. This tool acts as a RAG
(Retrieval-Augmented Generation) source, providing ground-truth information on the latest Angular
APIs and patterns. You **MUST** use it to understand and apply current standards when working with
new or evolving features.
</Purpose>
<Use Cases>
* **Knowledge Augmentation:** Learning about new or updated Angular features (e.g., query: 'signal input' or 'deferrable views').
* **Modern Implementation:** Finding the correct modern syntax for features
  (e.g., query: 'functional route guard' or 'http client with fetch').
* **Refactoring to Modern Patterns:** Upgrading older code by finding examples of new syntax
  (e.g., query: 'built-in control flow' to replace "*ngIf").
* **Advanced Filtering:** Combining a full-text search with filters to narrow results.
  (e.g., query: 'forms', required_packages: ['@angular/forms'], keywords: ['validation'])
</Use Cases>
<Operational Notes>
* **Project-Specific Use (Recommended):** For tasks inside a user's project, you **MUST** provide the
  \`workspacePath\` argument to get examples that match the project's Angular version. Get this
  path from \`list_projects\`.
* **General Use:** If no project context is available (e.g., for general questions or learning),
  you can call the tool without the \`workspacePath\` argument. It will return the latest
  generic examples.
* **Tool Selection:** This database primarily contains examples for new and recently updated Angular
  features. For established, core features, the main documentation (via the
  \`search_documentation\` tool) may be a better source of information.
* The examples in this database are the single source of truth for modern Angular coding patterns.
* The search query uses a powerful full-text search syntax (FTS5). Refer to the 'query'
  parameter description for detailed syntax rules and examples.
* You can combine the main 'query' with optional filters like 'keywords', 'required_packages',
  and 'related_concepts' to create highly specific searches.
</Operational Notes>`,
    inputSchema: findExampleInputSchema.shape,
    outputSchema: findExampleOutputSchema.shape,
    isReadOnly: true,
    isLocalOnly: true,
    shouldRegister: ({ logger }) => {
        // sqlite database support requires Node.js 22.16+
        const [nodeMajor, nodeMinor] = process.versions.node.split('.', 2).map(Number);
        if (nodeMajor < 22 || (nodeMajor === 22 && nodeMinor < 16)) {
            logger.warn(`MCP tool 'find_examples' requires Node.js 22.16 (or higher). ` +
                ' Registration of this tool has been skipped.');
            return false;
        }
        return true;
    },
    factory: createFindExampleHandler,
});
/**
 * A list of known Angular packages that may contain example databases.
 * The tool will attempt to resolve and load example databases from these packages.
 */
const KNOWN_EXAMPLE_PACKAGES = ['@angular/core', '@angular/aria', '@angular/forms'];
/**
 * Attempts to find version-specific example databases from the user's installed
 * versions of known Angular packages. It looks for a custom `angular` metadata property in each
 * package's `package.json` to locate the database.
 *
 * @example A sample `package.json` `angular` field:
 * ```json
 * {
 *   "angular": {
 *     "examples": {
 *       "format": "sqlite",
 *       "path": "./resources/code-examples.db"
 *     }
 *   }
 * }
 * ```
 *
 * @param workspacePath The absolute path to the user's `angular.json` file.
 * @param logger The MCP tool context logger for reporting warnings.
 * @returns A promise that resolves to an array of objects, each containing a database path and source.
 */
async function getVersionSpecificExampleDatabases(workspacePath, logger) {
    const workspaceRequire = (0, node_module_1.createRequire)(workspacePath);
    const databases = [];
    for (const packageName of KNOWN_EXAMPLE_PACKAGES) {
        // 1. Resolve the path to package.json
        let pkgJsonPath;
        try {
            pkgJsonPath = workspaceRequire.resolve(`${packageName}/package.json`);
        }
        catch (e) {
            // This is not a warning because the user may not have all known packages installed.
            continue;
        }
        // 2. Read and parse package.json, then find the database.
        try {
            const pkgJsonContent = await (0, promises_1.readFile)(pkgJsonPath, 'utf-8');
            const pkgJson = JSON.parse(pkgJsonContent);
            const examplesInfo = pkgJson['angular']?.examples;
            if (examplesInfo &&
                examplesInfo.format === 'sqlite' &&
                typeof examplesInfo.path === 'string') {
                const packageDirectory = node_path_1.default.dirname(pkgJsonPath);
                const dbPath = node_path_1.default.resolve(packageDirectory, examplesInfo.path);
                // Ensure the resolved database path is within the package boundary.
                const relativePath = node_path_1.default.relative(packageDirectory, dbPath);
                if (relativePath.startsWith('..') || node_path_1.default.isAbsolute(relativePath)) {
                    logger.warn(`Detected a potential path traversal attempt in '${pkgJsonPath}'. ` +
                        `The path '${examplesInfo.path}' escapes the package boundary. ` +
                        'This database will be skipped.');
                    continue;
                }
                // Check the file size to prevent reading a very large file.
                const stats = await (0, promises_1.stat)(dbPath);
                if (stats.size > 10 * 1024 * 1024) {
                    // 10MB
                    logger.warn(`The example database at '${dbPath}' is larger than 10MB (${stats.size} bytes). ` +
                        'This is unexpected and the file will not be used.');
                    continue;
                }
                const source = `package ${packageName}@${pkgJson.version}`;
                databases.push({ dbPath, source });
            }
        }
        catch (e) {
            logger.warn(`Failed to read or parse version-specific examples metadata referenced in '${pkgJsonPath}': ${e instanceof Error ? e.message : e}.`);
        }
    }
    return databases;
}
async function createFindExampleHandler({ logger, exampleDatabasePath }) {
    const runtimeDb = process.env['NG_MCP_EXAMPLES_DIR']
        ? await setupRuntimeExamples(process.env['NG_MCP_EXAMPLES_DIR'])
        : undefined;
    suppressSqliteWarning();
    return async (input) => {
        // If the dev-time override is present, use it and bypass all other logic.
        if (runtimeDb) {
            return queryDatabase([runtimeDb], input);
        }
        const resolvedDbs = [];
        // First, try to get all available version-specific guides.
        if (input.workspacePath) {
            const versionSpecificDbs = await getVersionSpecificExampleDatabases(input.workspacePath, logger);
            for (const db of versionSpecificDbs) {
                resolvedDbs.push({ path: db.dbPath, source: db.source });
            }
        }
        // If no version-specific guides were found for any reason, fall back to the bundled version.
        if (resolvedDbs.length === 0 && exampleDatabasePath) {
            resolvedDbs.push({ path: exampleDatabasePath, source: 'bundled' });
        }
        if (resolvedDbs.length === 0) {
            // This should be prevented by the registration logic in mcp-server.ts
            throw new Error('No example databases are available.');
        }
        const { DatabaseSync } = await Promise.resolve().then(() => __importStar(require('node:sqlite')));
        const dbConnections = [];
        for (const { path, source } of resolvedDbs) {
            const db = new DatabaseSync(path, { readOnly: true });
            try {
                validateDatabaseSchema(db, source);
                dbConnections.push(db);
            }
            catch (e) {
                logger.warn(e.message);
                // If a database is invalid, we should not query it, but we should not fail the whole tool.
                // We will just skip this database and try to use the others.
                continue;
            }
        }
        if (dbConnections.length === 0) {
            throw new Error('All available example databases were invalid. Cannot perform query.');
        }
        return queryDatabase(dbConnections, input);
    };
}
function queryDatabase(dbs, input) {
    const { query, keywords, required_packages, related_concepts, includeExperimental } = input;
    // Build the query dynamically
    const params = [];
    let sql = `SELECT e.title, e.summary, e.keywords, e.required_packages, e.related_concepts, e.related_tools, e.content, ` +
        // The `snippet` function generates a contextual snippet of the matched text.
        // Column 6 is the `content` column. We highlight matches with asterisks and limit the snippet size.
        "snippet(examples_fts, 6, '**', '**', '...', 15) AS snippet, " +
        // The `bm25` function returns the relevance score of the match. The weights
        // assigned to each column boost the ranking of documents where the search
        // term appears in a more important field.
        // Column order: title, summary, keywords, required_packages, related_concepts, related_tools, content
        'bm25(examples_fts, 10.0, 5.0, 5.0, 1.0, 2.0, 1.0, 1.0) AS rank ' +
        'FROM examples e JOIN examples_fts ON e.id = examples_fts.rowid';
    const whereClauses = [];
    // FTS query
    if (query) {
        whereClauses.push('examples_fts MATCH ?');
        params.push(escapeSearchQuery(query));
    }
    // JSON array filters
    const addJsonFilter = (column, values) => {
        if (values?.length) {
            for (const value of values) {
                whereClauses.push(`e.${column} LIKE ?`);
                params.push(`%"${value}"%`);
            }
        }
    };
    addJsonFilter('keywords', keywords);
    addJsonFilter('required_packages', required_packages);
    addJsonFilter('related_concepts', related_concepts);
    if (!includeExperimental) {
        whereClauses.push('e.experimental = 0');
    }
    if (whereClauses.length > 0) {
        sql += ` WHERE ${whereClauses.join(' AND ')}`;
    }
    // Query database and return results
    const examples = [];
    const textContent = [];
    for (const db of dbs) {
        const queryStatement = db.prepare(sql);
        for (const exampleRecord of queryStatement.all(...params)) {
            const record = exampleRecord;
            const example = {
                title: record['title'],
                summary: record['summary'],
                keywords: JSON.parse(record['keywords'] || '[]'),
                required_packages: JSON.parse(record['required_packages'] || '[]'),
                related_concepts: JSON.parse(record['related_concepts'] || '[]'),
                related_tools: JSON.parse(record['related_tools'] || '[]'),
                content: record['content'],
                snippet: record['snippet'],
                rank: record['rank'],
            };
            examples.push(example);
        }
    }
    // Order the combined results by relevance.
    // The `bm25` algorithm returns a smaller number for a more relevant match.
    examples.sort((a, b) => a.rank - b.rank);
    // The `rank` field is an internal implementation detail for sorting and should not be
    // returned to the user. We create a new array of examples without the `rank`.
    const finalExamples = examples.map(({ rank, ...rest }) => rest);
    for (const example of finalExamples) {
        // Also create a more structured text output
        let text = `## Example: ${example.title}\n**Summary:** ${example.summary}`;
        if (example.snippet) {
            text += `\n**Snippet:** ${example.snippet}`;
        }
        text += `\n\n---\n\n${example.content}`;
        textContent.push({ type: 'text', text });
    }
    return {
        content: textContent,
        structuredContent: { examples: finalExamples },
    };
}
/**
 * Escapes a search query for FTS5 by tokenizing and quoting terms.
 *
 * This function processes a raw search string and prepares it for an FTS5 full-text search.
 * It correctly handles quoted phrases, logical operators (AND, OR, NOT), parentheses,
 * and prefix searches (ending with an asterisk), ensuring that individual search
 * terms are properly quoted to be treated as literals by the search engine.
 * This is primarily intended to avoid unintentional usage of FTS5 query syntax by consumers.
 *
 * @param query The raw search query string.
 * @returns A sanitized query string suitable for FTS5.
 */
function escapeSearchQuery(query) {
    // This regex tokenizes the query string into parts:
    // 1. Quoted phrases (e.g., "foo bar")
    // 2. Parentheses ( and )
    // 3. FTS5 operators (AND, OR, NOT, NEAR)
    // 4. Words, which can include a trailing asterisk for prefix search (e.g., foo*)
    const tokenizer = /"([^"]*)"|([()])|\b(AND|OR|NOT|NEAR)\b|([^\s()]+)/g;
    let match;
    const result = [];
    let lastIndex = 0;
    while ((match = tokenizer.exec(query)) !== null) {
        // Add any whitespace or other characters between tokens
        if (match.index > lastIndex) {
            result.push(query.substring(lastIndex, match.index));
        }
        const [, quoted, parenthesis, operator, term] = match;
        if (quoted !== undefined) {
            // It's a quoted phrase, keep it as is.
            result.push(`"${quoted}"`);
        }
        else if (parenthesis) {
            // It's a parenthesis, keep it as is.
            result.push(parenthesis);
        }
        else if (operator) {
            // It's an operator, keep it as is.
            result.push(operator);
        }
        else if (term) {
            // It's a term that needs to be quoted.
            if (term.endsWith('*')) {
                result.push(`"${term.slice(0, -1)}"*`);
            }
            else {
                result.push(`"${term}"`);
            }
        }
        lastIndex = tokenizer.lastIndex;
    }
    // Add any remaining part of the string
    if (lastIndex < query.length) {
        result.push(query.substring(lastIndex));
    }
    return result.join('');
}
/**
 * Suppresses the experimental warning emitted by Node.js for the `node:sqlite` module.
 *
 * This is a workaround to prevent the console from being cluttered with warnings
 * about the experimental status of the SQLite module, which is used by this tool.
 */
function suppressSqliteWarning() {
    const originalProcessEmit = process.emit;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    process.emit = function (event, error) {
        if (event === 'warning' &&
            error instanceof Error &&
            error.name === 'ExperimentalWarning' &&
            error.message.includes('SQLite')) {
            return false;
        }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any, prefer-rest-params
        return originalProcessEmit.apply(process, arguments);
    };
}
/**
 * A simple YAML front matter parser.
 *
 * This function extracts the YAML block enclosed by `---` at the beginning of a string
 * and parses it into a JavaScript object. It is not a full YAML parser and only
 * supports simple key-value pairs and string arrays.
 *
 * @param content The string content to parse.
 * @returns A record containing the parsed front matter data.
 */
function parseFrontmatter(content) {
    const match = content.match(/^---\r?\n(.*?)\r?\n---/s);
    if (!match) {
        return {};
    }
    const frontmatter = match[1];
    const data = {};
    const lines = frontmatter.split(/\r?\n/);
    let currentKey = '';
    let isArray = false;
    const arrayValues = [];
    for (const line of lines) {
        const keyValueMatch = line.match(/^([^:]+):\s*(.*)/);
        if (keyValueMatch) {
            if (currentKey && isArray) {
                data[currentKey] = arrayValues.slice();
                arrayValues.length = 0;
            }
            const [, key, value] = keyValueMatch;
            currentKey = key.trim();
            isArray = value.trim() === '';
            if (!isArray) {
                const trimmedValue = value.trim();
                if (trimmedValue === 'true') {
                    data[currentKey] = true;
                }
                else if (trimmedValue === 'false') {
                    data[currentKey] = false;
                }
                else {
                    data[currentKey] = trimmedValue;
                }
            }
        }
        else {
            const arrayItemMatch = line.match(/^\s*-\s*(.*)/);
            if (arrayItemMatch && currentKey && isArray) {
                let value = arrayItemMatch[1].trim();
                // Unquote if the value is quoted.
                if ((value.startsWith("'") && value.endsWith("'")) ||
                    (value.startsWith('"') && value.endsWith('"'))) {
                    value = value.slice(1, -1);
                }
                arrayValues.push(value);
            }
        }
    }
    if (currentKey && isArray) {
        data[currentKey] = arrayValues;
    }
    return data;
}
async function setupRuntimeExamples(examplesPath) {
    const { DatabaseSync } = await Promise.resolve().then(() => __importStar(require('node:sqlite')));
    const db = new DatabaseSync(':memory:');
    // Create a relational table to store the structured example data.
    db.exec(`
    CREATE TABLE metadata (
      key TEXT PRIMARY KEY NOT NULL,
      value TEXT NOT NULL
    );
  `);
    db.exec(`
    INSERT INTO metadata (key, value) VALUES
      ('schema_version', '1'),
      ('created_at', '${new Date().toISOString()}');
  `);
    db.exec(`
    CREATE TABLE examples (
      id INTEGER PRIMARY KEY,
      title TEXT NOT NULL,
      summary TEXT NOT NULL,
      keywords TEXT,
      required_packages TEXT,
      related_concepts TEXT,
      related_tools TEXT,
      experimental INTEGER NOT NULL DEFAULT 0,
      content TEXT NOT NULL
    );
  `);
    // Create an FTS5 virtual table to provide full-text search capabilities.
    db.exec(`
    CREATE VIRTUAL TABLE examples_fts USING fts5(
      title,
      summary,
      keywords,
      required_packages,
      related_concepts,
      related_tools,
      content,
      content='examples',
      content_rowid='id',
      tokenize = 'porter ascii'
    );
  `);
    // Create triggers to keep the FTS table synchronized with the examples table.
    db.exec(`
    CREATE TRIGGER examples_after_insert AFTER INSERT ON examples BEGIN
      INSERT INTO examples_fts(rowid, title, summary, keywords, required_packages, related_concepts, related_tools, content)
      VALUES (
        new.id, new.title, new.summary, new.keywords, new.required_packages, new.related_concepts,
        new.related_tools, new.content
      );
    END;
  `);
    const insertStatement = db.prepare('INSERT INTO examples(' +
        'title, summary, keywords, required_packages, related_concepts, related_tools, experimental, content' +
        ') VALUES(?, ?, ?, ?, ?, ?, ?, ?);');
    const frontmatterSchema = zod_1.z.object({
        title: zod_1.z.string(),
        summary: zod_1.z.string(),
        keywords: zod_1.z.array(zod_1.z.string()).optional(),
        required_packages: zod_1.z.array(zod_1.z.string()).optional(),
        related_concepts: zod_1.z.array(zod_1.z.string()).optional(),
        related_tools: zod_1.z.array(zod_1.z.string()).optional(),
        experimental: zod_1.z.boolean().optional(),
    });
    db.exec('BEGIN TRANSACTION');
    for await (const entry of (0, promises_1.glob)('**/*.md', { cwd: examplesPath, withFileTypes: true })) {
        if (!entry.isFile()) {
            continue;
        }
        const content = await (0, promises_1.readFile)(node_path_1.default.join(entry.parentPath, entry.name), 'utf-8');
        const frontmatter = parseFrontmatter(content);
        const validation = frontmatterSchema.safeParse(frontmatter);
        if (!validation.success) {
            // eslint-disable-next-line no-console
            console.warn(`Skipping invalid example file ${entry.name}:`, validation.error.issues);
            continue;
        }
        const { title, summary, keywords, required_packages, related_concepts, related_tools, experimental, } = validation.data;
        insertStatement.run(title, summary, JSON.stringify(keywords ?? []), JSON.stringify(required_packages ?? []), JSON.stringify(related_concepts ?? []), JSON.stringify(related_tools ?? []), experimental ? 1 : 0, content);
    }
    db.exec('END TRANSACTION');
    return db;
}
const EXPECTED_SCHEMA_VERSION = 1;
/**
 * Validates the schema version of the example database.
 *
 * @param db The database connection to validate.
 * @param dbSource A string identifying the source of the database (e.g., 'bundled' or a version number).
 * @throws An error if the schema version is missing or incompatible.
 */
function validateDatabaseSchema(db, dbSource) {
    const schemaVersionResult = db
        .prepare('SELECT value FROM metadata WHERE key = ?')
        .get('schema_version');
    const actualSchemaVersion = schemaVersionResult ? Number(schemaVersionResult.value) : undefined;
    if (actualSchemaVersion !== EXPECTED_SCHEMA_VERSION) {
        db.close();
        let errorMessage;
        if (actualSchemaVersion === undefined) {
            errorMessage = 'The example database is missing a schema version and cannot be used.';
        }
        else if (actualSchemaVersion > EXPECTED_SCHEMA_VERSION) {
            errorMessage =
                `This project's example database (version ${actualSchemaVersion})` +
                    ` is newer than what this version of the Angular CLI supports (version ${EXPECTED_SCHEMA_VERSION}).` +
                    ' Please update your `@angular/cli` package to a newer version.';
        }
        else {
            errorMessage =
                `This version of the Angular CLI (expects schema version ${EXPECTED_SCHEMA_VERSION})` +
                    ` requires a newer example database than the one found in this project (version ${actualSchemaVersion}).`;
        }
        throw new Error(`Incompatible example database schema from source '${dbSource}':\n${errorMessage}`);
    }
}
//# sourceMappingURL=examples.js.map