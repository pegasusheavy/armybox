"use strict";
/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.injectKarmaReporter = injectKarmaReporter;
const test_files_1 = require("../../utils/test-files");
const results_1 = require("../application/results");
const LATEST_BUILD_FILES_TOKEN = 'angularLatestBuildFiles';
function injectKarmaReporter(buildOptions, buildIterator, karmaConfig, controller) {
    const reporterName = 'angular-progress-notifier';
    class ProgressNotifierReporter {
        emitter;
        latestBuildFiles;
        static $inject = ['emitter', LATEST_BUILD_FILES_TOKEN];
        // Needed for the karma reporter interface, see https://github.com/angular/angular-cli/issues/31629
        adapters = [];
        constructor(emitter, latestBuildFiles) {
            this.emitter = emitter;
            this.latestBuildFiles = latestBuildFiles;
            this.startWatchingBuild();
        }
        startWatchingBuild() {
            void (async () => {
                // This is effectively "for await of but skip what's already consumed".
                let isDone = false; // to mark the loop condition as "not constant".
                while (!isDone) {
                    const { done, value: buildOutput } = await buildIterator.next();
                    if (done) {
                        isDone = true;
                        break;
                    }
                    if (buildOutput.kind === results_1.ResultKind.Failure) {
                        controller.enqueue({ success: false, message: 'Build failed' });
                    }
                    else if (buildOutput.kind === results_1.ResultKind.Incremental ||
                        buildOutput.kind === results_1.ResultKind.Full) {
                        if (buildOutput.kind === results_1.ResultKind.Full) {
                            this.latestBuildFiles.files = buildOutput.files;
                        }
                        else {
                            this.latestBuildFiles.files = {
                                ...this.latestBuildFiles.files,
                                ...buildOutput.files,
                            };
                        }
                        await (0, test_files_1.writeTestFiles)(buildOutput.files, buildOptions.outputPath);
                        this.emitter.refreshFiles();
                    }
                }
            })();
        }
        onRunComplete = function (_browsers, results) {
            if (results.exitCode === 0) {
                controller.enqueue({ success: true });
            }
            else {
                controller.enqueue({ success: false });
            }
        };
    }
    karmaConfig.reporters ??= [];
    karmaConfig.reporters.push(reporterName);
    karmaConfig.plugins ??= [];
    karmaConfig.plugins.push({
        [`reporter:${reporterName}`]: [
            'factory',
            Object.assign((...args) => new ProgressNotifierReporter(...args), ProgressNotifierReporter),
        ],
    });
}
//# sourceMappingURL=progress-reporter.js.map