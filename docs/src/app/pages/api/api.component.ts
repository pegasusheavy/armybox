import { Component } from '@angular/core';

@Component({
  selector: 'app-api',
  standalone: true,
  template: `
    <div class="py-16 px-6">
      <div class="max-w-4xl mx-auto">
        <!-- Header -->
        <div class="mb-12">
          <h1 class="text-4xl font-bold text-army-900 mb-4">API Reference</h1>
          <p class="text-army-600">
            Multi-call binary architecture and <code class="bg-army-100 px-1 rounded">#[no_std]</code> Rust library.
          </p>
        </div>

        <!-- Multi-call Binary -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Multi-call Binary</h2>

          <p class="text-army-600 mb-6">
            armybox uses a multi-call binary architecture where a single executable
            provides all utilities. The applet is determined by the invocation name.
          </p>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3"># Direct invocation</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> armybox ls -la</div>
              <div><span class="text-army-500">$</span> armybox cat /etc/passwd</div>
            </div>
            <div class="text-army-400 mt-4 mb-3"># Via symlink</div>
            <div class="text-army-100">
              <div><span class="text-army-500">$</span> ln -s /usr/local/bin/armybox /usr/local/bin/ls</div>
              <div><span class="text-army-500">$</span> ls -la  <span class="text-army-500"># invokes armybox ls</span></div>
            </div>
          </div>
        </section>

        <!-- CLI Options -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">CLI Options</h2>

          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-army-200 text-left">
                  <th class="py-3 pr-4 font-semibold text-army-900">Option</th>
                  <th class="py-3 font-semibold text-army-900">Description</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-army-100">
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">--help, -h</code></td>
                  <td class="py-3 text-army-700">Display help information</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">--list, -l</code></td>
                  <td class="py-3 text-army-700">List all available applets</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">--version, -V</code></td>
                  <td class="py-3 text-army-700">Display version information</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">--install &lt;dir&gt;</code></td>
                  <td class="py-3 text-army-700">Create symlinks for all applets in directory</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        <!-- Architecture -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">#[no_std] Architecture</h2>

          <p class="text-army-600 mb-6">
            armybox is built entirely with <code class="text-xs bg-army-100 px-1 py-0.5 rounded">#[no_std]</code>,
            meaning it doesn't use the Rust standard library. Instead, it relies only on:
          </p>

          <div class="grid md:grid-cols-2 gap-6 mb-6">
            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-3">Dependencies</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-center gap-2">
                  <code class="text-xs bg-army-100 px-1 py-0.5 rounded">libc</code> Raw syscall interface
                </li>
                <li class="flex items-center gap-2">
                  <code class="text-xs bg-army-100 px-1 py-0.5 rounded">alloc</code> Heap allocation (Vec, String)
                </li>
                <li class="flex items-center gap-2">
                  <code class="text-xs bg-army-100 px-1 py-0.5 rounded">core</code> Rust core library
                </li>
              </ul>
            </div>

            <div class="p-4 border border-army-200 rounded-lg bg-white">
              <h3 class="font-medium text-army-900 mb-3">Components</h3>
              <ul class="space-y-2 text-sm text-army-700">
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span> Custom panic handler
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span> Custom allocator (libc malloc)
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span> Raw I/O via libc
                </li>
                <li class="flex items-center gap-2">
                  <span class="text-camo-olive">✓</span> No main runtime
                </li>
              </ul>
            </div>
          </div>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3">// Source structure</div>
            <pre class="text-army-100">src/
├── lib.rs          <span class="text-army-500"># Library entry (#[no_std])</span>
├── main.rs         <span class="text-army-500"># Binary entry (#[no_main])</span>
├── io.rs           <span class="text-army-500"># Raw I/O via libc</span>
├── sys.rs          <span class="text-army-500"># System utilities</span>
└── applets/
    ├── mod.rs      <span class="text-army-500"># Applet registry</span>
    ├── file.rs     <span class="text-army-500"># cat, cp, ls, mkdir...</span>
    ├── text.rs     <span class="text-army-500"># echo, head, tail, wc...</span>
    ├── system.rs   <span class="text-army-500"># uname, ps, id, kill...</span>
    └── misc.rs     <span class="text-army-500"># test, clear, sleep...</span></pre>
          </div>
        </section>

        <!-- Rust Library API -->
        <section class="mb-16">
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Rust Library API</h2>

          <p class="text-army-600 mb-6">
            armybox can be used as a <code class="text-xs bg-army-100 px-1 py-0.5 rounded">#[no_std]</code>
            library in embedded environments.
          </p>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto mb-6">
            <div class="text-army-400 mb-3">// Cargo.toml</div>
            <pre class="text-army-100">[dependencies]
armybox = {{ '{' }} version = "0.2", default-features = false, features = ["alloc"] {{ '}' }}</pre>
          </div>

          <div class="bg-army-900 rounded-xl p-5 font-mono text-sm overflow-x-auto">
            <div class="text-army-400 mb-3">// Example usage</div>
            <pre class="text-army-100"><span class="text-purple-400">#![no_std]</span>
<span class="text-purple-400">extern crate</span> alloc;

<span class="text-purple-400">use</span> armybox::{{ '{' }} run_applet, is_applet, applets {{ '}' }};

<span class="text-army-500">// Check if an applet exists</span>
<span class="text-purple-400">let</span> exists = is_applet(<span class="text-yellow-400">b"echo"</span>);

<span class="text-army-500">// List all applets</span>
<span class="text-purple-400">for</span> (name, _) <span class="text-purple-400">in</span> applets::APPLETS {{ '{' }}
    <span class="text-army-500">// name is &amp;[u8]</span>
{{ '}' }}

<span class="text-army-500">// Run an applet (takes C-style argc/argv)</span>
<span class="text-purple-400">let</span> exit_code = run_applet(<span class="text-yellow-400">b"echo"</span>, argc, argv);</pre>
          </div>
        </section>

        <!-- Available Features -->
        <section>
          <h2 class="text-2xl font-semibold text-army-900 mb-6">Cargo Features</h2>

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
                  <td class="py-3 text-army-700">Heap allocation (Vec, String, Box)</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4"><code class="text-xs bg-army-100 px-1 py-0.5 rounded">std</code></td>
                  <td class="py-3 pr-4 text-army-400">—</td>
                  <td class="py-3 text-army-700">Standard library (for testing only)</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </div>
  `
})
export class ApiComponent {}
