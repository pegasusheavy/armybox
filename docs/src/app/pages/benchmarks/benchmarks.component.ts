import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-benchmarks',
  standalone: true,
  imports: [RouterLink],
  template: `
    <div class="py-16 px-6">
      <div class="max-w-4xl mx-auto">
        <!-- Header -->
        <div class="mb-12">
          <h1 class="text-4xl font-bold text-army-900 mb-4">Benchmarks</h1>
          <p class="text-army-600">
            Performance comparison against BusyBox 1.37.0 on Linux x86_64 (WSL2).
            Tests run on real workloads ranging from 100K to 1M lines.
          </p>
          <p class="text-sm text-army-500 mt-2">
            Looking for feature comparisons? See the <a routerLink="/comparison" class="text-camo-olive hover:underline font-medium">full comparison page</a>.
          </p>

          <!-- Key Findings Banner -->
          <div class="mt-6 p-4 rounded-lg bg-gradient-to-r from-camo-olive/10 to-camo-brown/10 border border-camo-olive/20">
            <div class="flex items-center gap-4 flex-wrap">
              <div class="flex items-center gap-2">
                <span class="text-2xl">🚀</span>
                <span class="font-semibold text-army-900">Key Finding:</span>
              </div>
              <div class="text-sm text-army-700">
                <strong class="text-green-600">6 applets faster than BusyBox</strong> (grep 2.17x, wc 2x, sort 1.98x, awk 1.83x, tr 1.5x, sed 1.33x)
              </div>
              <div class="text-sm text-army-600">|</div>
              <div class="text-sm text-army-700">
                <strong>8+ applets</strong> at parity
              </div>
            </div>
          </div>
        </div>

        <!-- Summary Card -->
        <section class="mb-16">
          <div class="grid md:grid-cols-4 gap-6">
            <div class="p-6 rounded-xl border border-army-200 bg-white text-center">
              <div class="text-4xl font-bold text-camo-olive mb-2">201</div>
              <div class="text-sm text-army-600">Total Applets</div>
            </div>
            <div class="p-6 rounded-xl border border-army-200 bg-white text-center">
              <div class="text-4xl font-bold text-army-900 mb-2">311 KB</div>
              <div class="text-sm text-army-600">Release Binary</div>
            </div>
            <div class="p-6 rounded-xl border border-army-200 bg-white text-center">
              <div class="text-4xl font-bold text-green-600 mb-2">118 KB</div>
              <div class="text-sm text-army-600">UPX Compressed</div>
            </div>
            <div class="p-6 rounded-xl border border-army-200 bg-white text-center">
              <div class="text-4xl font-bold text-army-900 mb-2">~1.75 KB</div>
              <div class="text-sm text-army-600">Per Applet</div>
            </div>
          </div>
        </section>

        <!-- Faster than BusyBox -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">✅ Faster than BusyBox</h2>

          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Applet</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">Test</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">armybox</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">BusyBox</th>
                  <th class="py-3 font-semibold text-green-600">Speedup</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4 text-army-700">grep</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">1M lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.06s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.13s</td>
                  <td class="py-3 font-semibold text-green-600">2.17x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">wc</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.02s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.04s</td>
                  <td class="py-3 font-semibold text-green-600">2.00x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">sort -n</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">1M numbers</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.57s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">1.13s</td>
                  <td class="py-3 font-semibold text-green-600">1.98x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">awk</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K×10 iter</td>
                  <td class="py-3 pr-4 font-mono text-army-900">1.89s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">3.46s</td>
                  <td class="py-3 font-semibold text-green-600">1.83x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">tr</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.04s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.06s</td>
                  <td class="py-3 font-semibold text-green-600">1.50x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">sed</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.03s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.04s</td>
                  <td class="py-3 font-semibold text-green-600">1.33x faster</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Near Parity -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">⚖️ Near Parity (~1.0x)</h2>

          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Applet</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">Test</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">armybox</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">BusyBox</th>
                  <th class="py-3 font-semibold text-army-600">Ratio</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4 text-army-700">true</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">1000 invocations</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.28s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.30s</td>
                  <td class="py-3 text-green-600 font-medium">1.07x faster</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">cat</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">head/tail</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">ls</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100 files</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">md5sum</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">4MB file</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">uniq</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">cut</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">100K lines</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.02s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.02s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">dd</td>
                  <td class="py-3 pr-4 text-army-500 text-xs">4MB copy</td>
                  <td class="py-3 pr-4 font-mono text-army-900">0.01s</td>
                  <td class="py-3 pr-4 font-mono text-army-500">0.01s</td>
                  <td class="py-3 text-army-600">equal</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Startup Analysis -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Startup Time Analysis</h2>

          <div class="p-6 rounded-xl border border-army-200 bg-white">
            <p class="text-army-600 mb-4">
              Measured with 1000 invocations of <code class="bg-army-100 px-1 rounded">true</code>.
              With <code class="bg-army-100 px-1 rounded">#[no_std]</code>, armybox has minimal runtime initialization:
            </p>

            <div class="grid md:grid-cols-3 gap-4 mb-6">
              <div class="p-4 bg-camo-olive/10 rounded-lg border border-camo-olive/30">
                <div class="text-2xl font-bold text-camo-olive">~0.28ms</div>
                <div class="text-sm text-army-600">armybox (no_std)</div>
                <div class="text-xs text-green-600 mt-1">✓ Fastest</div>
              </div>
              <div class="p-4 bg-army-50 rounded-lg">
                <div class="text-2xl font-bold text-army-900">~0.30ms</div>
                <div class="text-sm text-army-600">BusyBox</div>
              </div>
              <div class="p-4 bg-army-50 rounded-lg">
                <div class="text-2xl font-bold text-army-900">~0.30ms</div>
                <div class="text-sm text-army-600">Toybox</div>
              </div>
            </div>

            <p class="text-sm text-army-500">
              By eliminating the Rust standard library runtime, armybox achieves startup times
              <strong>7% faster</strong> than BusyBox. The binary search applet dispatch adds
              only nanoseconds of overhead for 201 applets.
            </p>
          </div>
        </section>

        <!-- Optimizations -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Optimization Techniques</h2>

          <div class="grid md:grid-cols-2 gap-4">
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">Binary Search Dispatch</h3>
              <p class="text-sm text-army-600">O(log n) applet lookup using sorted static array - no heap allocation</p>
            </div>
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">Direct libc Calls</h3>
              <p class="text-sm text-army-600">All I/O through direct syscalls - no std::io overhead</p>
            </div>
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">Zero-Copy Parsing</h3>
              <p class="text-sm text-army-600">Arguments parsed as &amp;[u8] slices without String allocation</p>
            </div>
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">Stack Buffers</h3>
              <p class="text-sm text-army-600">8KB stack buffers for I/O - heap only when required</p>
            </div>
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">LTO + Strip</h3>
              <p class="text-sm text-army-600">Fat LTO, size optimization (opt-level=z), symbol stripping</p>
            </div>
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-2">Custom Allocator</h3>
              <p class="text-sm text-army-600">libc::malloc-based global allocator - no jemalloc bloat</p>
            </div>
          </div>
        </section>

        <!-- Run Benchmarks -->
        <section>
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Run Benchmarks</h2>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># Compare against BusyBox</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> ./scripts/benchmark-compare.sh</div>
            </div>
            <div class="text-army-400 mt-4 mb-3"># Run Criterion benchmarks</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> cargo bench</div>
            </div>
            <div class="text-army-400 mt-4 mb-3"># Quick comparison with hyperfine</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> hyperfine './target/release/armybox true' 'busybox true'</div>
            </div>
          </div>
        </section>
      </div>
    </div>
  `
})
export class BenchmarksComponent {}
