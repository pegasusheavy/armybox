import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [RouterLink],
  template: `
    <!-- Hero Section with Camo Pattern -->
    <section class="py-24 px-6 relative overflow-hidden">
      <!-- Background camo shapes -->
      <div class="absolute inset-0 opacity-10">
        <div class="absolute top-10 left-10 w-64 h-48 rounded-full bg-camo-olive blur-3xl"></div>
        <div class="absolute top-40 right-20 w-80 h-60 rounded-full bg-camo-forest blur-3xl"></div>
        <div class="absolute bottom-20 left-1/3 w-72 h-56 rounded-full bg-camo-brown blur-3xl"></div>
      </div>

      <div class="max-w-4xl mx-auto text-center relative z-10">
        <!-- Military badge -->
        <div class="inline-flex items-center gap-3 px-4 py-2 rounded mb-8 border-2 border-camo-olive" style="background: linear-gradient(135deg, rgba(74, 93, 35, 0.9) 0%, rgba(45, 64, 34, 0.9) 100%);">
          <span class="w-3 h-3 bg-camo-tan rounded-full animate-pulse"></span>
          <span class="text-camo-sand font-bold uppercase tracking-wider text-sm">v0.3.0 — 293 APPLETS — 108KB / 54KB UPX — 100% TOYBOX COMPATIBLE</span>
        </div>

        <h1 class="text-5xl md:text-7xl font-stencil text-army-900 mb-6 tracking-widest">
          <span class="text-camo-gradient">#[NO_STD]</span><br>
          <span class="text-camo-olive">UNIX UTILS</span>
        </h1>

        <p class="text-xl text-army-700 mb-10 max-w-2xl mx-auto text-balance font-medium">
          A tactical BusyBox/Toybox clone built in pure Rust with <code class="bg-army-100 px-1 rounded">#[no_std]</code>.
          Memory-safe, combat-ready, incredibly tiny. <strong class="text-camo-olive">100% Toybox compatible.</strong>
        </p>

        <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
          <a routerLink="/building" class="btn btn-primary px-8 py-3 text-base">
            ⚡ Deploy Now
          </a>
          <a routerLink="/applets" class="btn btn-secondary px-8 py-3 text-base">
            📋 View Arsenal
          </a>
          <a href="https://github.com/pegasusheavy/armybox" target="_blank" class="btn btn-secondary px-8 py-3 text-base">
            🔗 GitHub
          </a>
        </div>
      </div>
    </section>

    <!-- Install Section with Terminal Look -->
    <section class="py-16 px-6">
      <div class="max-w-4xl mx-auto">
        <div class="rounded-lg overflow-hidden border-2 border-camo-olive" style="box-shadow: 6px 6px 0 rgba(45, 64, 34, 0.4);">
          <!-- Terminal header -->
          <div class="px-4 py-3 flex items-center gap-3" style="background: linear-gradient(135deg, #4a5d23 0%, #2d4022 100%);">
            <div class="flex gap-2">
              <div class="w-3 h-3 rounded-full bg-camo-brown"></div>
              <div class="w-3 h-3 rounded-full bg-camo-khaki"></div>
              <div class="w-3 h-3 rounded-full bg-camo-tan"></div>
            </div>
            <span class="text-camo-sand/70 text-sm font-mono">mission-control</span>
          </div>
          <!-- Terminal body -->
          <div class="p-6 font-mono text-sm" style="background: linear-gradient(135deg, #2d2b1f 0%, #1a1910 100%);">
            <div class="flex items-center gap-2 text-camo-khaki mb-4">
              <span class="text-camo-olive">#</span> Quick deployment
            </div>
            <div class="text-camo-sand">
              <span class="text-camo-tan">$</span> cargo build --release
            </div>
            <div class="text-camo-sand mt-2">
              <span class="text-camo-tan">$</span> ./target/release/armybox --install /usr/local/bin
            </div>
            <div class="text-camo-olive mt-4 text-xs">
              # 293 utilities deployed. 108KB mission-ready. 54KB with UPX.
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Camo divider -->
    <div class="divider-camo max-w-6xl mx-auto"></div>

    <!-- Features Grid -->
    <section class="py-24 px-6">
      <div class="max-w-6xl mx-auto">
        <h2 class="text-3xl font-stencil text-army-900 mb-4 text-center tracking-widest">TACTICAL ADVANTAGES</h2>
        <p class="text-army-600 text-center mb-16 max-w-2xl mx-auto">
          Engineered from the ground up in Rust with <code class="bg-army-100 px-1 rounded">#[no_std]</code> for embedded systems, containers, and hostile environments.
        </p>

        <div class="grid md:grid-cols-3 gap-8">
          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">Incredibly Tiny</h3>
            <p class="text-sm text-army-700">
              <strong>108KB</strong> release binary, <strong>~54KB</strong> with UPX.
              That's ~380 bytes per applet — 24x more efficient than BusyBox.
            </p>
          </div>

          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">True #[no_std]</h3>
            <p class="text-sm text-army-700">
              No Rust standard library. Only <code class="bg-army-100 px-0.5 rounded text-xs">libc</code> and <code class="bg-army-100 px-0.5 rounded text-xs">alloc</code>.
              Perfect for embedded and constrained environments.
            </p>
          </div>

          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">Memory Safe</h3>
            <p class="text-sm text-army-700">
              Rust's ownership system eliminates buffer overflows,
              use-after-free, and data race vulnerabilities.
            </p>
          </div>

          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">100% Toybox Compatible</h3>
            <p class="text-sm text-army-700">
              All 238 Toybox commands plus 55 additional utilities.
              Drop-in replacement for any Toybox deployment.
            </p>
          </div>

          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">Single Payload</h3>
            <p class="text-sm text-army-700">
              All 293 utilities in one tiny binary.
              Perfect for containers (FROM scratch), initramfs, rescue.
            </p>
          </div>

          <div class="card">
            <div class="icon-camo mb-4">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"/>
              </svg>
            </div>
            <h3 class="font-bold text-army-900 mb-2 uppercase tracking-wider">Android Native</h3>
            <p class="text-sm text-army-700">
              First-class Android support with Bionic libc compatibility.
              Deploy to ARM64, ARMv7, or x86_64 Android devices.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- Size Comparison Section -->
    <section class="py-24 px-6" style="background: linear-gradient(135deg, rgba(74, 93, 35, 0.08) 0%, rgba(92, 64, 51, 0.05) 100%);">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-3xl font-stencil text-army-900 mb-4 text-center tracking-widest">SIZE EFFICIENCY</h2>
        <p class="text-army-600 text-center mb-12">
          Comparison on Linux x86_64 — BusyBox 1.37, Toybox 0.8.11.
        </p>

        <div class="grid md:grid-cols-3 gap-6 mb-12">
          <div class="card border-camo-olive">
            <div class="flex items-center gap-2 mb-4">
              <div class="w-8 h-8 rounded border border-camo-olive flex items-center justify-center" style="background: linear-gradient(135deg, #4a5d23 0%, #2d4022 100%);">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" class="w-6 h-6">
                  <g transform="translate(4, 2) scale(0.00267, -0.00267) translate(0, -6000)" fill="#d4c5a9">
                    <path d="M4544 5991 c-52 -13 -133 -63 -186 -115 -87 -85 -125 -194 -135 -389 l-6 -107 40 -39 c57 -54 67 -75 53 -115 -14 -42 -45 -70 -95 -86 -22 -7 -60 -25 -85 -40 -25 -16 -89 -44 -143 -64 -53 -19 -118 -47 -144 -62 -100 -58 -181 -166 -315 -419 -41 -77 -124 -225 -184 -330 -159 -275 -174 -317 -138 -388 53 -104 213 -237 345 -286 l27 -10 -5 -108 c-4 -97 -2 -115 21 -182 l26 -74 -64 -171 c-214 -568 -295 -745 -446 -971 -96 -143 -147 -239 -221 -412 -26 -62 -58 -126 -71 -142 -12 -17 -31 -41 -40 -54 -9 -13 -84 -78 -165 -143 -81 -65 -184 -150 -228 -190 -48 -43 -107 -84 -147 -103 -96 -46 -140 -108 -163 -235 -6 -31 -3 -43 16 -69 71 -98 218 -127 454 -88 433 72 991 22 1333 -121 86 -36 163 -102 257 -223 81 -102 98 -118 165 -149 159 -73 465 -85 755 -30 250 48 413 132 446 232 26 77 -24 204 -115 295 -97 97 -262 187 -341 187 -24 0 -45 9 -67 29 -32 29 -33 31 -32 108 2 115 62 289 132 381 23 30 26 44 27 120 1 77 6 97 40 172 78 177 145 414 145 517 0 118 -85 342 -260 688 -69 138 -131 273 -137 300 -6 28 -27 113 -48 190 -29 109 -36 150 -31 189 3 27 8 51 10 54 3 2 26 -14 51 -36 110 -96 427 -294 510 -317 28 -8 80 -17 116 -20 l65 -7 50 -65 c27 -35 70 -94 96 -131 53 -75 80 -90 171 -91 49 -1 62 3 82 23 13 14 33 27 45 31 11 3 23 17 26 31 3 13 10 24 15 24 6 0 24 22 41 50 17 27 39 52 50 56 10 3 30 23 45 45 14 21 31 40 37 40 6 1 47 3 91 4 44 1 96 6 116 10 23 4 84 -1 166 -14 151 -25 308 -28 316 -8 3 8 8 39 12 70 7 51 5 56 -16 71 -63 40 -305 102 -514 131 -128 17 -899 168 -1050 205 -49 12 -102 29 -117 36 -46 24 -47 37 -12 349 12 110 15 203 11 350 -6 257 -13 279 -105 344 -37 25 -78 58 -92 72 -14 14 -40 34 -58 43 -39 20 -112 84 -112 98 0 5 5 6 10 3 22 -13 51 8 84 63 19 31 46 68 60 81 14 14 26 31 26 38 0 8 16 23 35 34 40 24 42 34 19 125 -10 37 -13 69 -8 73 5 4 40 16 77 26 l68 18 -7 31 c-13 60 -95 288 -123 345 -55 108 -176 197 -301 221 -60 11 -209 12 -256 1z"/>
                  </g>
                </svg>
              </div>
              <h3 class="font-bold text-camo-olive uppercase tracking-wider">Armybox</h3>
            </div>
            <ul class="space-y-3 text-sm">
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Binary Size</span>
                <strong class="text-camo-olive">108 KB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">UPX Compressed</span>
                <strong class="text-camo-olive">~54 KB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Applets</span>
                <strong class="text-army-900">293 (100% Toybox + extras)</strong>
              </li>
              <li class="flex justify-between py-2">
                <span class="text-army-800 font-medium">Size per Applet</span>
                <strong class="text-camo-olive">~380 bytes</strong>
              </li>
            </ul>
          </div>

          <div class="card">
            <div class="flex items-center gap-2 mb-4">
              <div class="w-8 h-8 rounded border border-amber-300 bg-amber-50 flex items-center justify-center">
                <span class="text-amber-600 text-lg">🧸</span>
              </div>
              <h3 class="font-bold text-amber-600 uppercase tracking-wider">Toybox</h3>
            </div>
            <ul class="space-y-3 text-sm">
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Binary Size</span>
                <strong class="text-army-500">~500 KB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">UPX Compressed</span>
                <strong class="text-army-500">~200 KB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Applets</span>
                <strong class="text-army-500">238</strong>
              </li>
              <li class="flex justify-between py-2">
                <span class="text-army-800 font-medium">Size per Applet</span>
                <strong class="text-army-500">~2.1 KB</strong>
              </li>
            </ul>
          </div>

          <div class="card">
            <div class="flex items-center gap-2 mb-4">
              <div class="w-8 h-8 rounded border border-army-300 bg-army-100 flex items-center justify-center">
                <span class="text-army-600 text-lg">📦</span>
              </div>
              <h3 class="font-bold text-army-500 uppercase tracking-wider">BusyBox</h3>
            </div>
            <ul class="space-y-3 text-sm">
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Binary Size</span>
                <strong class="text-army-500">2.4 MB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">UPX Compressed</span>
                <strong class="text-army-500">~1 MB</strong>
              </li>
              <li class="flex justify-between py-2 border-b border-camo-sand">
                <span class="text-army-800 font-medium">Applets</span>
                <strong class="text-army-500">274+</strong>
              </li>
              <li class="flex justify-between py-2">
                <span class="text-army-800 font-medium">Size per Applet</span>
                <strong class="text-army-500">~9 KB</strong>
              </li>
            </ul>
          </div>
        </div>

        <div class="p-4 rounded-lg text-center" style="background: rgba(74, 93, 35, 0.1);">
          <p class="text-camo-forest font-medium">
            Armybox is <strong>24x more efficient per applet</strong> than BusyBox and <strong>5.5x more efficient</strong> than Toybox!
          </p>
        </div>
      </div>
    </section>

    <!-- Comparison Table -->
    <section class="py-24 px-6">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-3xl font-stencil text-army-900 mb-4 text-center tracking-widest">INTEL COMPARISON</h2>
        <p class="text-army-600 text-center mb-12">
          See how armybox stacks up against BusyBox and Toybox.
        </p>

        <div class="overflow-x-auto rounded-lg border-2 border-camo-olive" style="box-shadow: 4px 4px 0 rgba(45, 64, 34, 0.3);">
          <table class="w-full text-sm">
            <thead>
              <tr>
                <th class="text-left">Feature</th>
                <th class="text-center">armybox</th>
                <th class="text-center">BusyBox</th>
                <th class="text-center">Toybox</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Language</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">Rust #[no_std]</td>
                <td class="py-3 px-4 text-center text-army-500">C</td>
                <td class="py-3 px-4 text-center text-army-500">C</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Memory Safety</td>
                <td class="py-3 px-4 text-center text-camo-olive font-bold">✓ Compile-time</td>
                <td class="py-3 px-4 text-center text-army-400">— Manual</td>
                <td class="py-3 px-4 text-center text-army-400">— Manual</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Binary Size</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">108 KB</td>
                <td class="py-3 px-4 text-center text-army-500">2.4 MB</td>
                <td class="py-3 px-4 text-center text-army-500">~500 KB</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">UPX Compressed</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">~54 KB</td>
                <td class="py-3 px-4 text-center text-army-500">~1 MB</td>
                <td class="py-3 px-4 text-center text-army-500">~200 KB</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Applets</td>
                <td class="py-3 px-4 text-center font-bold text-army-900">293</td>
                <td class="py-3 px-4 text-center text-army-500">274+</td>
                <td class="py-3 px-4 text-center text-army-500">238</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Toybox Compatible</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">✓ 100%</td>
                <td class="py-3 px-4 text-center text-army-400">Partial</td>
                <td class="py-3 px-4 text-center text-army-500">N/A</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Size/Applet</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">~380 bytes</td>
                <td class="py-3 px-4 text-center text-army-500">~9 KB</td>
                <td class="py-3 px-4 text-center text-army-500">~2.1 KB</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">License</td>
                <td class="py-3 px-4 text-center font-bold text-camo-olive">MIT/Apache-2.0</td>
                <td class="py-3 px-4 text-center text-army-500">GPL v2</td>
                <td class="py-3 px-4 text-center text-army-500">0BSD</td>
              </tr>
              <tr>
                <td class="py-3 px-4 text-army-800 font-medium">Embedded Ready</td>
                <td class="py-3 px-4 text-center text-camo-olive font-bold">✓</td>
                <td class="py-3 px-4 text-center text-camo-olive">✓</td>
                <td class="py-3 px-4 text-center text-camo-olive">✓</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="mt-8 text-center">
          <a routerLink="/comparison" class="btn btn-secondary px-8 py-3 text-base">
            📊 Full Comparison →
          </a>
        </div>
      </div>
    </section>
  `
})
export class HomeComponent {}
