/**
 * @license
 * Copyright Google LLC All Rights Reserved.
 *
 * Use of this source code is governed by an MIT-style license that can be
 * found in the LICENSE file at https://angular.dev/license
 */
import type { ApplicationBuilderInternalOptions } from '../../../application/options';
import { NormalizedUnitTestBuilderOptions } from '../../options';
import { RunnerOptions } from '../api';
export declare function getVitestBuildOptions(options: NormalizedUnitTestBuilderOptions, baseBuildOptions: Partial<ApplicationBuilderInternalOptions>): Promise<RunnerOptions>;
