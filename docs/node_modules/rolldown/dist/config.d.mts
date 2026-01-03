import { n as ConfigExport, t as defineConfig } from "./shared/define-config-DdWK1ZyG.mjs";
import "./shared/binding-BHtM7anm.mjs";

//#region src/utils/load-config.d.ts
declare function loadConfig(configPath: string): Promise<ConfigExport>;
//#endregion
//#region src/config.d.ts
declare const VERSION: string;
//#endregion
export { VERSION, defineConfig, loadConfig };