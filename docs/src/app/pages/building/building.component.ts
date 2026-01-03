import { Component } from '@angular/core';

@Component({
  selector: 'app-building',
  standalone: true,
  template: `
    <div class="py-16 px-6">
      <div class="max-w-4xl mx-auto">
        <!-- Header -->
        <div class="mb-12">
          <h1 class="text-4xl font-bold text-army-900 mb-4">Building</h1>
          <p class="text-army-600">
            Build armybox from source. The <code class="bg-army-100 px-1 rounded">#[no_std]</code> design produces incredibly small binaries.
          </p>
        </div>

        <!-- Binary Size Summary -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Binary Sizes</h2>

          <div class="overflow-x-auto mb-6">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Build Type</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">Size</th>
                  <th class="py-3 font-semibold text-army-900">Command</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4 text-army-700">Release</td>
                  <td class="py-3 pr-4 font-mono text-camo-olive font-bold">74 KB</td>
                  <td class="py-3"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">cargo build --release</code></td>
                </tr>
                <tr class="bg-army-50">
                  <td class="py-3 pr-4 text-army-900 font-medium">UPX Compressed</td>
                  <td class="py-3 pr-4 font-mono text-green-700 font-bold">36 KB</td>
                  <td class="py-3"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">upx --best armybox</code></td>
                </tr>
              </tbody>
            </table>
          </div>

          <p class="text-sm text-army-600">
            All builds include <strong>57 applets</strong>. That's approximately <strong>1.3 KB per applet</strong>.
          </p>
        </section>

        <!-- Prerequisites -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Prerequisites</h2>

          <div class="prose-army">
            <ul class="space-y-3 text-army-700">
              <li class="flex items-start gap-3">
                <span class="text-army-400 mt-1">•</span>
                <span><strong>Rust 2024</strong> (1.85+) - install via <a href="https://rustup.rs" class="text-army-600 hover:text-army-900">rustup</a></span>
              </li>
              <li class="flex items-start gap-3">
                <span class="text-army-400 mt-1">•</span>
                <span><strong>upx</strong> (optional) for smallest builds: <code class="text-sm bg-army-100 px-1 py-0.5 rounded">apt install upx-ucl</code></span>
              </li>
              <li class="flex items-start gap-3">
                <span class="text-army-400 mt-1">•</span>
                <span><strong>musl-tools</strong> (optional) for static builds: <code class="text-sm bg-army-100 px-1 py-0.5 rounded">apt install musl-tools</code></span>
              </li>
            </ul>
          </div>
        </section>

        <!-- Quick Start -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Quick Start</h2>

          <div class="space-y-6">
            <div>
              <h3 class="text-lg font-medium text-army-900 mb-3">Clone and build</h3>
              <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
                <div class="text-army-100">
                  <div><span class="text-army-500">$</span> git clone https://github.com/PegasusHeavyIndustries/armybox</div>
                  <div><span class="text-army-500">$</span> cd armybox</div>
                  <div><span class="text-army-500">$</span> cargo build --release</div>
                </div>
              </div>
            </div>

            <div>
              <h3 class="text-lg font-medium text-army-900 mb-3">Compress with UPX (optional)</h3>
              <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
                <div class="text-army-100">
                  <div><span class="text-army-500">$</span> upx --best target/release/armybox</div>
                  <div class="text-army-400 mt-2"># 74KB → 36KB</div>
                </div>
              </div>
            </div>

            <div>
              <h3 class="text-lg font-medium text-army-900 mb-3">Install to system</h3>
              <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
                <div class="text-army-100">
                  <div><span class="text-army-500">$</span> sudo install -m 755 target/release/armybox /usr/local/bin/</div>
                  <div><span class="text-army-500">$</span> armybox --install /usr/local/bin</div>
                  <div class="text-army-400 mt-2"># Creates symlinks for all 57 applets</div>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Size Optimizations -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Size Optimizations</h2>

          <p class="text-army-600 mb-6">
            Armybox achieves its tiny size through <code class="bg-army-100 px-1 rounded">#[no_std]</code> and aggressive compiler optimizations.
          </p>

          <div class="grid md:grid-cols-2 gap-6 mb-6">
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-3">Cargo Profile</h3>
              <div class="bg-army-900 rounded-lg p-3 font-mono text-xs overflow-x-auto">
                <div class="text-army-100">
                  <div>[profile.release]</div>
                  <div>opt-level = "z"       <span class="text-army-500"># Size</span></div>
                  <div>lto = "fat"           <span class="text-army-500"># Full LTO</span></div>
                  <div>codegen-units = 1     <span class="text-army-500"># Better opt</span></div>
                  <div>panic = "abort"       <span class="text-army-500"># No unwind</span></div>
                  <div>strip = "symbols"     <span class="text-army-500"># Strip all</span></div>
                </div>
              </div>
            </div>

            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-3">Why So Small?</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span><strong>#[no_std]</strong> - No Rust stdlib</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span><strong>Direct libc</strong> - Raw syscalls</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span><strong>No runtime</strong> - #[no_main]</span>
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span>
                  <span><strong>panic=abort</strong> - No unwinding</span>
                </li>
              </ul>
            </div>
          </div>

          <div class="p-4 bg-army-100 rounded-lg border border-army-200">
            <p class="text-sm text-army-700">
              <strong class="text-army-900">UPX Compression:</strong> Using
              <code class="text-xs bg-army-200 px-1 py-0.5 rounded">upx --best</code>
              compresses the binary from 74KB to ~36KB (48% of original size) while remaining fully functional.
            </p>
          </div>
        </section>

        <!-- Linker Configuration -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Linker Configuration</h2>

          <p class="text-army-600 mb-6">
            The <code class="bg-army-100 px-1 rounded">.cargo/config.toml</code> configures linker flags for size optimization.
          </p>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># .cargo/config.toml</div>
            <div class="text-army-100 whitespace-pre">[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "link-arg=-Wl,--gc-sections",  <span class="text-army-500"># Remove unused</span>
    "-C", "link-arg=-Wl,--as-needed",    <span class="text-army-500"># Only needed libs</span>
    "-C", "link-arg=-Wl,-O2",            <span class="text-army-500"># Linker opt</span>
    "-C", "link-arg=-lc",                <span class="text-army-500"># Link libc</span>
]</div>
          </div>
        </section>

        <!-- Cross Compilation -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Cross Compilation</h2>

          <p class="text-army-600 mb-6">
            Build for different architectures using Rust's cross-compilation support.
          </p>

          <div class="overflow-x-auto mb-6">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Target</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">Command</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4 text-army-700">x86_64 Linux (musl)</td>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">cargo build --release --target x86_64-unknown-linux-musl</code></td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">ARM64 (musl)</td>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">cargo build --release --target aarch64-unknown-linux-musl</code></td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 text-army-700">ARM32 (musl)</td>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">cargo build --release --target armv7-unknown-linux-musleabihf</code></td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># Install target and build</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> rustup target add x86_64-unknown-linux-musl</div>
              <div><span class="text-army-500">$</span> cargo build --release --target x86_64-unknown-linux-musl</div>
            </div>
          </div>
        </section>

        <!-- Feature Flags -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Cargo Features</h2>

          <p class="text-army-600 mb-6">
            Armybox has minimal features since it's <code class="bg-army-100 px-1 rounded">#[no_std]</code>.
          </p>

          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Feature</th>
                  <th class="py-3 pr-4 font-semibold text-army-900">Default</th>
                  <th class="py-3 font-semibold text-army-900">Description</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">alloc</code></td>
                  <td class="py-3 pr-4 text-camo-olive font-bold">✓</td>
                  <td class="py-3 text-army-700">Heap allocation (Vec, String)</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">std</code></td>
                  <td class="py-3 pr-4 text-army-400">—</td>
                  <td class="py-3 text-army-700">Standard library (for testing)</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Docker -->
        <section>
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Docker</h2>

          <p class="text-army-600 mb-6">
            Use armybox as a minimal base for containers. Perfect for FROM scratch images.
          </p>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># Dockerfile (~100KB total image!)</div>
            <div class="text-army-100">
              <div>FROM scratch</div>
              <div>COPY target/release/armybox /bin/armybox</div>
              <div>RUN ["/bin/armybox", "--install", "/bin"]</div>
              <div>ENTRYPOINT ["/bin/sh"]</div>
            </div>
          </div>

          <div class="mt-6 bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># Build the image</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> cargo build --release</div>
              <div><span class="text-army-500">$</span> docker build -t myapp .</div>
              <div><span class="text-army-500">$</span> docker images myapp</div>
              <div class="text-army-400 mt-2">REPOSITORY   TAG       SIZE</div>
              <div class="text-army-400">myapp        latest    ~100KB</div>
            </div>
          </div>

          <div class="mt-6 p-4 bg-camo-olive/10 rounded-lg border border-camo-olive/30">
            <p class="text-sm text-camo-forest">
              <strong class="text-camo-olive">Note:</strong> With UPX compression, you can get Docker images under 50KB!
            </p>
          </div>
        </section>
      </div>
    </div>
  `
})
export class BuildingComponent {}
