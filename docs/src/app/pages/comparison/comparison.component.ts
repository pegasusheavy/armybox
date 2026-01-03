import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

interface ComparisonItem {
  feature: string;
  armybox: string;
  busybox: string;
  toybox: string;
  armyboxBest?: boolean;
}

@Component({
  selector: 'app-comparison',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="py-16 px-6">
      <div class="max-w-6xl mx-auto">
        <!-- Header -->
        <div class="mb-16 text-center">
          <h1 class="text-4xl md:text-5xl font-stencil text-army-900 mb-6 tracking-widest">
            <span class="text-camo-gradient">TACTICAL</span> COMPARISON
          </h1>
          <p class="text-army-600 max-w-2xl mx-auto text-lg">
            How does armybox stack up against the established Unix utility toolkits?
            See how we compare to BusyBox and Toybox across key metrics.
          </p>
        </div>

        <!-- Three-way Overview Cards -->
        <section class="mb-20">
          <div class="grid md:grid-cols-3 gap-6">
            <!-- Armybox -->
            <div class="card border-camo-olive relative overflow-hidden">
              <div class="absolute top-0 left-0 right-0 h-1 bg-camo-olive"></div>
              <div class="flex items-center gap-3 mb-4">
                <div class="w-10 h-10 rounded border-2 border-camo-olive flex items-center justify-center p-1" style="background: linear-gradient(135deg, #3d2b1f 0%, #2d2b1f 100%);">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="#d4c5a9" class="w-full h-full">
                    <rect x="10" y="1" width="12" height="3" rx="1"/>
                    <rect x="8" y="3" width="16" height="4" rx="1"/>
                    <circle cx="16" cy="11" r="3"/>
                    <path d="M11 14 L11 22 L21 22 L21 14 Q19 13 16 13 Q13 13 11 14 Z"/>
                  </svg>
                </div>
                <div>
                  <h3 class="font-stencil text-camo-olive text-lg tracking-wider">ARMYBOX</h3>
                  <span class="text-xs text-army-500 uppercase tracking-wider">Memory Safe</span>
                </div>
              </div>
              <ul class="space-y-2 text-sm">
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span class="text-army-700"><strong class="text-camo-olive">Rust #[no_std]</strong> — Memory safe by design</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span class="text-army-700"><strong class="text-camo-olive">108 KB</strong> binary (~54 KB UPX)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span class="text-army-700"><strong class="text-camo-olive">291</strong> applets (100% Toybox + 53 extras)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span class="text-army-700"><strong class="text-camo-olive">Android-native</strong> — Bionic libc support</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span class="text-army-700">MIT / Apache-2.0 dual license</span>
                </li>
              </ul>
            </div>

            <!-- BusyBox -->
            <div class="card relative overflow-hidden">
              <div class="absolute top-0 left-0 right-0 h-1 bg-army-400"></div>
              <div class="flex items-center gap-3 mb-4">
                <div class="w-10 h-10 rounded border-2 border-army-300 bg-army-100 flex items-center justify-center">
                  <span class="text-army-600 text-xl">📦</span>
                </div>
                <div>
                  <h3 class="font-bold text-army-700 text-lg">BusyBox</h3>
                  <span class="text-xs text-army-400 uppercase tracking-wider">The Original</span>
                </div>
              </div>
              <ul class="space-y-2 text-sm">
                <li class="flex items-center gap-2">
                  <span class="text-army-400">○</span>
                  <span class="text-army-600"><strong class="text-army-700">C</strong> — Manual memory management</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-army-400">○</span>
                  <span class="text-army-600"><strong class="text-army-700">2.4 MB</strong> binary (~1 MB UPX)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-army-400">○</span>
                  <span class="text-army-600"><strong class="text-army-700">274+</strong> applets (~9 KB each)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-army-400">○</span>
                  <span class="text-army-600">GPL v2 license</span>
                </li>
              </ul>
            </div>

            <!-- Toybox -->
            <div class="card relative overflow-hidden">
              <div class="absolute top-0 left-0 right-0 h-1 bg-amber-500"></div>
              <div class="flex items-center gap-3 mb-4">
                <div class="w-10 h-10 rounded border-2 border-amber-300 bg-amber-50 flex items-center justify-center">
                  <span class="text-amber-600 text-xl">🧸</span>
                </div>
                <div>
                  <h3 class="font-bold text-amber-700 text-lg">Toybox</h3>
                  <span class="text-xs text-amber-500 uppercase tracking-wider">Android's Choice</span>
                </div>
              </div>
              <ul class="space-y-2 text-sm">
                <li class="flex items-center gap-2">
                  <span class="text-amber-500">○</span>
                  <span class="text-army-600"><strong class="text-army-700">C</strong> — Manual memory management</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-amber-500">○</span>
                  <span class="text-army-600"><strong class="text-army-700">~500 KB</strong> binary (config dependent)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-amber-500">○</span>
                  <span class="text-army-600"><strong class="text-army-700">238</strong> applets (~2.1 KB each)</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-amber-500">○</span>
                  <span class="text-army-600">0BSD (Public Domain) license</span>
                </li>
              </ul>
            </div>
          </div>
        </section>

        <!-- Detailed Comparison Table -->
        <section class="mb-20">
          <h2 class="text-3xl font-stencil text-army-900 mb-8 text-center tracking-widest">DETAILED INTEL</h2>

          <div class="overflow-x-auto rounded-lg border-2 border-camo-olive" style="box-shadow: 4px 4px 0 rgba(45, 64, 34, 0.3);">
            <table class="w-full text-sm">
              <thead>
                <tr>
                  <th class="text-left w-1/4">Feature</th>
                  <th class="text-center w-1/4">
                    <span class="text-camo-sand">armybox</span>
                  </th>
                  <th class="text-center w-1/4">BusyBox</th>
                  <th class="text-center w-1/4">Toybox</th>
                </tr>
              </thead>
              <tbody>
                <tr *ngFor="let item of comparisonData">
                  <td class="py-3 px-4 text-army-800 font-medium">{{ item.feature }}</td>
                  <td class="py-3 px-4 text-center" [class.font-bold]="item.armyboxBest" [class.text-camo-olive]="item.armyboxBest">
                    {{ item.armybox }}
                  </td>
                  <td class="py-3 px-4 text-center text-army-600">{{ item.busybox }}</td>
                  <td class="py-3 px-4 text-center text-army-600">{{ item.toybox }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Size Efficiency Chart -->
        <section class="mb-20">
          <h2 class="text-3xl font-stencil text-army-900 mb-8 text-center tracking-widest">SIZE EFFICIENCY</h2>
          <p class="text-army-600 text-center mb-12">
            Bytes per applet — lower is better
          </p>

          <div class="max-w-3xl mx-auto space-y-6">
            <!-- Armybox Bar -->
            <div class="flex items-center gap-4">
              <div class="w-24 text-right font-bold text-camo-olive">armybox</div>
              <div class="flex-1 h-10 bg-army-100 rounded-lg overflow-hidden relative">
                <div class="absolute inset-y-0 left-0 bg-camo-olive rounded-lg flex items-center justify-end pr-3" style="width: 4%;">
                  <span class="text-camo-sand font-bold text-sm whitespace-nowrap ml-2">~380 B</span>
                </div>
              </div>
            </div>

            <!-- Toybox Bar -->
            <div class="flex items-center gap-4">
              <div class="w-24 text-right font-bold text-amber-600">Toybox</div>
              <div class="flex-1 h-10 bg-army-100 rounded-lg overflow-hidden relative">
                <div class="absolute inset-y-0 left-0 bg-amber-500 rounded-lg flex items-center justify-end pr-3" style="width: 23%;">
                  <span class="text-white font-bold text-sm">~2.1 KB</span>
                </div>
              </div>
            </div>

            <!-- BusyBox Bar -->
            <div class="flex items-center gap-4">
              <div class="w-24 text-right font-bold text-army-500">BusyBox</div>
              <div class="flex-1 h-10 bg-army-100 rounded-lg overflow-hidden relative">
                <div class="absolute inset-y-0 left-0 bg-army-400 rounded-lg flex items-center justify-end pr-3" style="width: 100%;">
                  <span class="text-white font-bold text-sm">~9 KB</span>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-12 p-6 rounded-lg text-center" style="background: rgba(74, 93, 35, 0.1);">
            <p class="text-camo-forest">
              armybox is <strong class="text-camo-olive">24x more efficient</strong> than BusyBox and
              <strong class="text-camo-olive">5.5x more efficient</strong> than Toybox per applet!
            </p>
          </div>
        </section>

        <!-- Toybox Deep Dive -->
        <section class="mb-20">
          <h2 class="text-3xl font-stencil text-army-900 mb-8 text-center tracking-widest">TOYBOX COMPATIBILITY</h2>

          <div class="grid md:grid-cols-2 gap-8">
            <div class="card">
              <h3 class="font-bold text-amber-700 mb-4 uppercase tracking-wider flex items-center gap-2">
                <span class="text-xl">🧸</span> 100% Toybox Compatible
              </h3>
              <p class="text-sm text-army-700 mb-4">
                Armybox includes <strong>all 238 Toybox commands</strong> plus 53 additional utilities.
                This makes armybox a drop-in replacement for any Toybox deployment while providing
                the memory safety guarantees of Rust.
              </p>
              <p class="text-sm text-army-700">
                <strong>Android Integration:</strong> Since Toybox is the default toolkit on Android 6.0+,
                armybox can replace it on billions of devices, bringing memory safety to the command line.
              </p>
            </div>

            <div class="card">
              <h3 class="font-bold text-camo-olive mb-4 uppercase tracking-wider flex items-center gap-2">
                <span class="text-xl">⚔️</span> Armybox Advantages
              </h3>
              <ul class="space-y-3 text-sm text-army-700">
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive mt-0.5">✓</span>
                  <span><strong>Memory Safety:</strong> Rust eliminates buffer overflows, use-after-free, and data races at compile time</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive mt-0.5">✓</span>
                  <span><strong>Smaller Binary:</strong> 108KB vs Toybox's ~500KB default build</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive mt-0.5">✓</span>
                  <span><strong>True #[no_std]:</strong> Works in embedded environments without a full libc</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive mt-0.5">✓</span>
                  <span><strong>More Applets:</strong> 291 vs 238 — includes init system, full networking, APK</span>
                </li>
              </ul>
            </div>
          </div>
        </section>

        <!-- Feature Matrix -->
        <section class="mb-20">
          <h2 class="text-3xl font-stencil text-army-900 mb-8 text-center tracking-widest">FEATURE MATRIX</h2>

          <div class="overflow-x-auto rounded-lg border-2 border-camo-olive" style="box-shadow: 4px 4px 0 rgba(45, 64, 34, 0.3);">
            <table class="w-full text-sm">
              <thead>
                <tr>
                  <th class="text-left">Category</th>
                  <th class="text-center">armybox</th>
                  <th class="text-center">BusyBox</th>
                  <th class="text-center">Toybox</th>
                </tr>
              </thead>
              <tbody>
                <tr *ngFor="let cat of featureMatrix">
                  <td class="py-3 px-4 text-army-800 font-medium">{{ cat.category }}</td>
                  <td class="py-3 px-4 text-center">
                    <span *ngIf="cat.armybox === 'full'" class="text-camo-olive font-bold">✓ Full</span>
                    <span *ngIf="cat.armybox === 'partial'" class="text-amber-600">◐ Partial</span>
                    <span *ngIf="cat.armybox === 'planned'" class="text-army-400">○ Planned</span>
                    <span *ngIf="cat.armybox === 'none'" class="text-army-300">—</span>
                  </td>
                  <td class="py-3 px-4 text-center">
                    <span *ngIf="cat.busybox === 'full'" class="text-army-600">✓ Full</span>
                    <span *ngIf="cat.busybox === 'partial'" class="text-army-500">◐ Partial</span>
                    <span *ngIf="cat.busybox === 'none'" class="text-army-300">—</span>
                  </td>
                  <td class="py-3 px-4 text-center">
                    <span *ngIf="cat.toybox === 'full'" class="text-army-600">✓ Full</span>
                    <span *ngIf="cat.toybox === 'partial'" class="text-army-500">◐ Partial</span>
                    <span *ngIf="cat.toybox === 'none'" class="text-army-300">—</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- When to Use What -->
        <section class="mb-20">
          <h2 class="text-3xl font-stencil text-army-900 mb-8 text-center tracking-widest">MISSION SELECTION</h2>
          <p class="text-army-600 text-center mb-12 max-w-2xl mx-auto">
            Choose the right tool for your mission. Each has strengths for different deployment scenarios.
          </p>

          <div class="grid md:grid-cols-3 gap-6">
            <div class="card border-camo-olive">
              <h3 class="font-bold text-camo-olive mb-4 uppercase tracking-wider">Choose Armybox When</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Memory safety is critical (security-focused deployments)</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Minimal binary size is the priority</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Building "FROM scratch" containers</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Embedded systems with strict constraints</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Replacing Toybox with memory safety</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-camo-olive">▸</span>
                  <span>Modern Rust toolchain available</span>
                </li>
              </ul>
            </div>

            <div class="card">
              <h3 class="font-bold text-army-600 mb-4 uppercase tracking-wider">Choose BusyBox When</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-start gap-2">
                  <span class="text-army-500">▸</span>
                  <span>Maximum applet coverage needed (274+)</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-army-500">▸</span>
                  <span>Legacy system compatibility required</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-army-500">▸</span>
                  <span>Complex shell scripting (ash/hush)</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-army-500">▸</span>
                  <span>Initramfs with advanced init</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-army-500">▸</span>
                  <span>GPL licensing is acceptable</span>
                </li>
              </ul>
            </div>

            <div class="card">
              <h3 class="font-bold text-amber-600 mb-4 uppercase tracking-wider">Choose Toybox When</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-start gap-2">
                  <span class="text-amber-500">▸</span>
                  <span>BSD/permissive licensing required</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-amber-500">▸</span>
                  <span>Android/AOSP compatibility needed</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-amber-500">▸</span>
                  <span>C toolchain, but avoiding GPL</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-amber-500">▸</span>
                  <span>No Rust toolchain available</span>
                </li>
                <li class="flex items-start gap-2">
                  <span class="text-amber-500">▸</span>
                  <span>Rob Landley's coding style preferred</span>
                </li>
              </ul>
            </div>
          </div>
        </section>

        <!-- Quick Comparison Summary -->
        <section class="py-12 px-8 rounded-lg mb-16" style="background: linear-gradient(135deg, rgba(74, 93, 35, 0.15) 0%, rgba(92, 64, 51, 0.08) 100%);">
          <h2 class="text-2xl font-stencil text-army-900 mb-8 text-center tracking-widest">TL;DR SUMMARY</h2>

          <div class="grid md:grid-cols-3 gap-8 text-center">
            <div>
              <div class="text-4xl mb-2">🔒</div>
              <h3 class="font-bold text-camo-olive mb-2">Armybox</h3>
              <p class="text-sm text-army-600">Smallest, safest, 100% Toybox compatible</p>
            </div>
            <div>
              <div class="text-4xl mb-2">📦</div>
              <h3 class="font-bold text-army-600 mb-2">BusyBox</h3>
              <p class="text-sm text-army-600">Most complete, battle-tested</p>
            </div>
            <div>
              <div class="text-4xl mb-2">🧸</div>
              <h3 class="font-bold text-amber-600 mb-2">Toybox</h3>
              <p class="text-sm text-army-600">BSD licensed, Android default</p>
            </div>
          </div>
        </section>
      </div>
    </div>
  `
})
export class ComparisonComponent {
  comparisonData: ComparisonItem[] = [
    { feature: 'Language', armybox: 'Rust #[no_std]', busybox: 'C', toybox: 'C', armyboxBest: true },
    { feature: 'Memory Safety', armybox: '✓ Compile-time', busybox: '— Manual', toybox: '— Manual', armyboxBest: true },
    { feature: 'Binary Size', armybox: '108 KB', busybox: '2.4 MB', toybox: '~500 KB', armyboxBest: true },
    { feature: 'UPX Compressed', armybox: '~54 KB', busybox: '~1 MB', toybox: '~200 KB', armyboxBest: true },
    { feature: 'Applet Count', armybox: '291', busybox: '274+', toybox: '238', armyboxBest: true },
    { feature: 'Toybox Compatible', armybox: '✓ 100%', busybox: 'Partial', toybox: 'N/A', armyboxBest: true },
    { feature: 'Size per Applet', armybox: '~380 bytes', busybox: '~9 KB', toybox: '~2.1 KB', armyboxBest: true },
    { feature: 'License', armybox: 'MIT/Apache-2.0', busybox: 'GPL v2', toybox: '0BSD' },
    { feature: 'Standard Library', armybox: 'None (no_std)', busybox: 'libc', toybox: 'libc' },
    { feature: 'Editor (vi)', armybox: '✓', busybox: '✓', toybox: '✓' },
    { feature: 'Shell Included', armybox: '✓ sh/ash/dash', busybox: 'ash/hush', toybox: 'sh' },
    { feature: 'Archiving', armybox: '✓ tar/gzip/xz/cpio', busybox: '✓ Full', toybox: '✓ Full' },
    { feature: 'Networking', armybox: '✓ Full (35+ cmds)', busybox: '✓ Full', toybox: '◐ Partial' },
    { feature: 'Init System', armybox: '✓ Yes', busybox: 'Yes', toybox: 'No', armyboxBest: true },
    { feature: 'Package Manager', armybox: '✓ APK (optional)', busybox: '—', toybox: '—', armyboxBest: true },
    { feature: 'Android Support', armybox: '✓ Native', busybox: 'Compatible', toybox: 'Native', armyboxBest: true },
    { feature: 'Embedded Ready', armybox: '✓', busybox: '✓', toybox: '✓' },
    { feature: 'Static Linking', armybox: '✓', busybox: '✓', toybox: '✓' },
    { feature: 'First Release', armybox: '2026', busybox: '1996', toybox: '2012' },
  ];

  featureMatrix = [
    { category: 'Core File Ops (ls, cp, mv, rm)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'Text Processing (cat, grep, sed, awk)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'System Info (uname, ps, df, free)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'Archiving (tar, gzip, bzip2, xz, cpio)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'Networking (wget, nc, ping, ifconfig)', armybox: 'full', busybox: 'full', toybox: 'partial' },
    { category: 'Shell (sh, ash, bash-like)', armybox: 'full', busybox: 'full', toybox: 'partial' },
    { category: 'Init System (init, runlevel)', armybox: 'full', busybox: 'full', toybox: 'none' },
    { category: 'Editors (vi)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'Process Management (kill, pkill, pgrep)', armybox: 'full', busybox: 'full', toybox: 'full' },
    { category: 'Module Management (insmod, rmmod)', armybox: 'full', busybox: 'full', toybox: 'partial' },
    { category: 'Package Management (apk)', armybox: 'full', busybox: 'none', toybox: 'none' },
    { category: 'Hardware (i2c, gpio, lspci, lsusb)', armybox: 'full', busybox: 'partial', toybox: 'full' },
  ];
}
