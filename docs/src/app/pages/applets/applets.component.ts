import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';

interface Applet {
  name: string;
  description: string;
  category: string;
}

@Component({
  selector: 'app-applets',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="py-16 px-6">
      <div class="max-w-6xl mx-auto">
        <!-- Header -->
        <div class="mb-12">
          <h1 class="text-4xl font-bold text-army-900 mb-4">Applets</h1>
          <p class="text-army-600 max-w-2xl">
            armybox includes <strong>{{ filteredApplets.length }}</strong> utilities organized by category.
            Each applet is accessible via symlink or as a subcommand.
            All applets are implemented in pure <code class="bg-army-100 px-1 rounded">#[no_std]</code> Rust.
          </p>
        </div>

        <!-- Search & Filter -->
        <div class="flex flex-col sm:flex-row gap-4 mb-8">
          <div class="relative flex-1">
            <input
              type="text"
              [(ngModel)]="searchQuery"
              (ngModelChange)="filterApplets()"
              placeholder="Search applets..."
              class="w-full px-4 py-2 pl-10 border border-army-200 rounded-lg bg-white text-army-900 placeholder-army-400 focus:outline-none focus:ring-2 focus:ring-army-200 focus:border-army-300"
            />
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-army-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
          </div>

          <select
            [(ngModel)]="selectedCategory"
            (ngModelChange)="filterApplets()"
            class="px-4 py-2 border border-army-200 rounded-lg bg-white text-army-900 focus:outline-none focus:ring-2 focus:ring-army-200 focus:border-army-300"
          >
            <option value="">All Categories</option>
            <option *ngFor="let cat of categories" [value]="cat">{{ cat }}</option>
          </select>
        </div>

        <!-- Stats -->
        <div class="flex flex-wrap gap-4 mb-8 text-sm">
          <span class="px-3 py-1 bg-camo-olive/10 text-camo-olive rounded-full font-medium">
            {{ filteredApplets.length }} applets
          </span>
          <span class="px-3 py-1 bg-army-100 text-army-700 rounded-full">
            324 KB binary
          </span>
          <span class="px-3 py-1 bg-army-100 text-army-700 rounded-full">
            ~1.6 KB per applet
          </span>
          <span class="px-3 py-1 bg-camo-olive/10 text-camo-olive rounded-full font-medium">
            ✓ Full networking stack
          </span>
        </div>

        <!-- Applets by Category -->
        <div class="space-y-12">
          <div *ngFor="let category of getUniqueCategories()" class="group">
            <h2 class="text-xl font-semibold text-army-900 mb-4 flex items-center gap-3">
              {{ category }}
              <span class="text-sm font-normal text-army-400">
                ({{ getAppletsByCategory(category).length }})
              </span>
            </h2>

            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              <div
                *ngFor="let applet of getAppletsByCategory(category)"
                class="flex items-start gap-3 p-3 rounded-lg border border-army-100 bg-white hover:border-camo-olive/50 transition-colors"
              >
                <code class="text-sm font-mono text-camo-olive font-bold min-w-[80px]">{{ applet.name }}</code>
                <span class="text-sm text-army-600 flex-1">{{ applet.description }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div *ngIf="filteredApplets.length === 0" class="text-center py-16">
          <p class="text-army-500">No applets found matching your search.</p>
        </div>
      </div>
    </div>
  `
})
export class AppletsComponent {
  searchQuery = '';
  selectedCategory = '';

  categories = [
    'File Operations',
    'Text Processing',
    'Archiving',
    'System Info',
    'Process Management',
    'Networking',
    'Checksums & Encoding',
    'Shell',
    'Init System',
    'Package Management',
    'Misc'
  ];

  applets: Applet[] = [
    // File Operations (30)
    { name: 'cat', description: 'Concatenate files', category: 'File Operations' },
    { name: 'cp', description: 'Copy files', category: 'File Operations' },
    { name: 'ls', description: 'List directory contents', category: 'File Operations' },
    { name: 'mkdir', description: 'Create directories', category: 'File Operations' },
    { name: 'rm', description: 'Remove files', category: 'File Operations' },
    { name: 'rmdir', description: 'Remove directories', category: 'File Operations' },
    { name: 'mv', description: 'Move files', category: 'File Operations' },
    { name: 'touch', description: 'Update timestamps', category: 'File Operations' },
    { name: 'pwd', description: 'Print working directory', category: 'File Operations' },
    { name: 'chmod', description: 'Change file mode', category: 'File Operations' },
    { name: 'chown', description: 'Change file owner', category: 'File Operations' },
    { name: 'chgrp', description: 'Change file group', category: 'File Operations' },
    { name: 'ln', description: 'Create links', category: 'File Operations' },
    { name: 'readlink', description: 'Read symbolic link', category: 'File Operations' },
    { name: 'stat', description: 'Display file status', category: 'File Operations' },
    { name: 'sync', description: 'Sync filesystems', category: 'File Operations' },
    { name: 'link', description: 'Create hard link', category: 'File Operations' },
    { name: 'unlink', description: 'Remove file', category: 'File Operations' },
    { name: 'cd', description: 'Change directory', category: 'File Operations' },
    { name: 'basename', description: 'Strip directory', category: 'File Operations' },
    { name: 'dirname', description: 'Strip filename', category: 'File Operations' },
    { name: 'dd', description: 'Convert and copy files', category: 'File Operations' },
    { name: 'file', description: 'Determine file type', category: 'File Operations' },
    { name: 'install', description: 'Copy and set attributes', category: 'File Operations' },
    { name: 'mkfifo', description: 'Create named pipe', category: 'File Operations' },
    { name: 'mknod', description: 'Create special files', category: 'File Operations' },
    { name: 'mktemp', description: 'Create temp file', category: 'File Operations' },
    { name: 'realpath', description: 'Resolve path', category: 'File Operations' },
    { name: 'shred', description: 'Secure delete', category: 'File Operations' },
    { name: 'split', description: 'Split file', category: 'File Operations' },
    { name: 'truncate', description: 'Shrink/extend file', category: 'File Operations' },

    // Text Processing (30)
    { name: 'echo', description: 'Display text', category: 'Text Processing' },
    { name: 'printf', description: 'Format output', category: 'Text Processing' },
    { name: 'yes', description: 'Output repeatedly', category: 'Text Processing' },
    { name: 'head', description: 'Output first lines', category: 'Text Processing' },
    { name: 'tail', description: 'Output last lines', category: 'Text Processing' },
    { name: 'wc', description: 'Word count', category: 'Text Processing' },
    { name: 'cut', description: 'Cut fields', category: 'Text Processing' },
    { name: 'tr', description: 'Translate characters', category: 'Text Processing' },
    { name: 'tee', description: 'Duplicate output', category: 'Text Processing' },
    { name: 'nl', description: 'Number lines', category: 'Text Processing' },
    { name: 'seq', description: 'Print sequences', category: 'Text Processing' },
    { name: 'sort', description: 'Sort lines', category: 'Text Processing' },
    { name: 'uniq', description: 'Filter duplicates', category: 'Text Processing' },
    { name: 'tac', description: 'Reverse file', category: 'Text Processing' },
    { name: 'awk', description: 'Pattern processing', category: 'Text Processing' },
    { name: 'comm', description: 'Compare files', category: 'Text Processing' },
    { name: 'dos2unix', description: 'DOS to Unix newlines', category: 'Text Processing' },
    { name: 'expand', description: 'Tabs to spaces', category: 'Text Processing' },
    { name: 'fmt', description: 'Reformat paragraphs', category: 'Text Processing' },
    { name: 'fold', description: 'Wrap lines', category: 'Text Processing' },
    { name: 'grep', description: 'Search patterns', category: 'Text Processing' },
    { name: 'paste', description: 'Merge lines', category: 'Text Processing' },
    { name: 'rev', description: 'Reverse lines', category: 'Text Processing' },
    { name: 'sed', description: 'Stream editor', category: 'Text Processing' },
    { name: 'strings', description: 'Print strings', category: 'Text Processing' },
    { name: 'unexpand', description: 'Spaces to tabs', category: 'Text Processing' },
    { name: 'unix2dos', description: 'Unix to DOS newlines', category: 'Text Processing' },

    // System Info (32)
    { name: 'arch', description: 'Print architecture', category: 'System Info' },
    { name: 'chroot', description: 'Change root directory', category: 'System Info' },
    { name: 'chvt', description: 'Change virtual terminal', category: 'System Info' },
    { name: 'date', description: 'Display date/time', category: 'System Info' },
    { name: 'df', description: 'Disk free space', category: 'System Info' },
    { name: 'dmesg', description: 'Kernel messages', category: 'System Info' },
    { name: 'du', description: 'Disk usage', category: 'System Info' },
    { name: 'env', description: 'Environment', category: 'System Info' },
    { name: 'fgconsole', description: 'Active VT', category: 'System Info' },
    { name: 'free', description: 'Memory usage', category: 'System Info' },
    { name: 'groups', description: 'Print groups', category: 'System Info' },
    { name: 'halt', description: 'Stop system', category: 'System Info' },
    { name: 'hostid', description: 'Host identifier', category: 'System Info' },
    { name: 'hostname', description: 'System hostname', category: 'System Info' },
    { name: 'id', description: 'User/group IDs', category: 'System Info' },
    { name: 'logger', description: 'Log messages', category: 'System Info' },
    { name: 'logname', description: 'Login name', category: 'System Info' },
    { name: 'lsmod', description: 'List modules', category: 'System Info' },
    { name: 'mount', description: 'Mount filesystems', category: 'System Info' },
    { name: 'mountpoint', description: 'Check mount point', category: 'System Info' },
    { name: 'nice', description: 'Run with priority', category: 'System Info' },
    { name: 'nohup', description: 'Ignore hangups', category: 'System Info' },
    { name: 'nproc', description: 'CPU count', category: 'System Info' },
    { name: 'poweroff', description: 'Power off system', category: 'System Info' },
    { name: 'printenv', description: 'Print environment', category: 'System Info' },
    { name: 'reboot', description: 'Reboot system', category: 'System Info' },
    { name: 'swapoff', description: 'Disable swap', category: 'System Info' },
    { name: 'swapon', description: 'Enable swap', category: 'System Info' },
    { name: 'sysctl', description: 'System params', category: 'System Info' },
    { name: 'timeout', description: 'Run with timeout', category: 'System Info' },
    { name: 'tty', description: 'Print terminal', category: 'System Info' },
    { name: 'umount', description: 'Unmount filesystems', category: 'System Info' },
    { name: 'uname', description: 'System information', category: 'System Info' },
    { name: 'uptime', description: 'System uptime', category: 'System Info' },
    { name: 'users', description: 'Logged in users', category: 'System Info' },
    { name: 'w', description: 'Who is logged in', category: 'System Info' },
    { name: 'who', description: 'Who is logged in', category: 'System Info' },
    { name: 'whoami', description: 'Current user', category: 'System Info' },

    // Process Management (11)
    { name: 'kill', description: 'Send signals', category: 'Process Management' },
    { name: 'killall', description: 'Kill by name', category: 'Process Management' },
    { name: 'pgrep', description: 'Find processes', category: 'Process Management' },
    { name: 'pidof', description: 'Find PID by name', category: 'Process Management' },
    { name: 'pkill', description: 'Kill by pattern', category: 'Process Management' },
    { name: 'ps', description: 'Process status', category: 'Process Management' },
    { name: 'pwdx', description: 'Process working dir', category: 'Process Management' },
    { name: 'renice', description: 'Change priority', category: 'Process Management' },
    { name: 'setsid', description: 'New session', category: 'Process Management' },

    // Checksums & Encoding (7)
    { name: 'base64', description: 'Base64 encode/decode', category: 'Checksums & Encoding' },
    { name: 'hexdump', description: 'Hex dump file', category: 'Checksums & Encoding' },
    { name: 'md5sum', description: 'MD5 checksum', category: 'Checksums & Encoding' },
    { name: 'od', description: 'Octal dump', category: 'Checksums & Encoding' },
    { name: 'xxd', description: 'Hex dump/reverse', category: 'Checksums & Encoding' },

    // Networking (35) - Full networking stack
    { name: 'arp', description: 'Manipulate ARP cache', category: 'Networking' },
    { name: 'arping', description: 'Send ARP requests', category: 'Networking' },
    { name: 'brctl', description: 'Ethernet bridge administration', category: 'Networking' },
    { name: 'ether-wake', description: 'Send Wake-on-LAN magic packet', category: 'Networking' },
    { name: 'ftpget', description: 'Download file via FTP', category: 'Networking' },
    { name: 'ftpput', description: 'Upload file via FTP', category: 'Networking' },
    { name: 'host', description: 'DNS lookup utility', category: 'Networking' },
    { name: 'ifconfig', description: 'Configure network interface', category: 'Networking' },
    { name: 'ifdown', description: 'Bring down network interface', category: 'Networking' },
    { name: 'ifup', description: 'Bring up network interface', category: 'Networking' },
    { name: 'ip', description: 'Show/manipulate routing, devices, interfaces', category: 'Networking' },
    { name: 'ipaddr', description: 'Protocol address management', category: 'Networking' },
    { name: 'ipcalc', description: 'IP address calculator', category: 'Networking' },
    { name: 'iplink', description: 'Network device configuration', category: 'Networking' },
    { name: 'ipneigh', description: 'Neighbour/ARP tables management', category: 'Networking' },
    { name: 'iproute', description: 'Routing table management', category: 'Networking' },
    { name: 'iprule', description: 'Routing policy database', category: 'Networking' },
    { name: 'nameif', description: 'Name network interfaces by MAC', category: 'Networking' },
    { name: 'nc', description: 'Arbitrary TCP/UDP connections', category: 'Networking' },
    { name: 'netcat', description: 'Alias for nc', category: 'Networking' },
    { name: 'netstat', description: 'Network statistics', category: 'Networking' },
    { name: 'nslookup', description: 'Query DNS servers', category: 'Networking' },
    { name: 'ping', description: 'Send ICMP ECHO_REQUEST', category: 'Networking' },
    { name: 'ping6', description: 'Send IPv6 ICMP ECHO_REQUEST', category: 'Networking' },
    { name: 'route', description: 'Show/manipulate routing table', category: 'Networking' },
    { name: 'slattach', description: 'Attach serial line to network', category: 'Networking' },
    { name: 'ss', description: 'Socket statistics', category: 'Networking' },
    { name: 'telnet', description: 'Telnet client', category: 'Networking' },
    { name: 'tftp', description: 'TFTP client', category: 'Networking' },
    { name: 'traceroute', description: 'Print route packets take', category: 'Networking' },
    { name: 'traceroute6', description: 'IPv6 traceroute', category: 'Networking' },
    { name: 'tunctl', description: 'Create/delete TUN/TAP devices', category: 'Networking' },
    { name: 'vconfig', description: 'VLAN configuration', category: 'Networking' },
    { name: 'wget', description: 'Non-interactive network downloader', category: 'Networking' },

    // Additional System (15)
    { name: 'blkid', description: 'Locate/print block device attributes', category: 'System Info' },
    { name: 'chrt', description: 'Manipulate real-time attributes', category: 'System Info' },
    { name: 'dnsdomainname', description: 'Show DNS domain name', category: 'System Info' },
    { name: 'flock', description: 'Manage file locks', category: 'System Info' },
    { name: 'fsync', description: 'Synchronize file state', category: 'System Info' },
    { name: 'insmod', description: 'Insert kernel module', category: 'System Info' },
    { name: 'ionice', description: 'Set I/O scheduling class', category: 'System Info' },
    { name: 'losetup', description: 'Set up loop devices', category: 'System Info' },
    { name: 'modprobe', description: 'Add/remove kernel modules', category: 'System Info' },
    { name: 'pivot_root', description: 'Change root filesystem', category: 'System Info' },
    { name: 'readahead', description: 'Preload files into cache', category: 'System Info' },
    { name: 'rfkill', description: 'Control wireless devices', category: 'System Info' },
    { name: 'rmmod', description: 'Remove kernel module', category: 'System Info' },
    { name: 'taskset', description: 'Set/get CPU affinity', category: 'System Info' },

    // File Operations additions
    { name: 'patch', description: 'Apply a unified diff', category: 'File Operations' },
    { name: 'xargs', description: 'Build commands from stdin', category: 'File Operations' },

    // Shell (3)
    { name: 'sh', description: 'POSIX-compliant shell', category: 'Shell' },
    { name: 'ash', description: 'Almquist shell (alias for sh)', category: 'Shell' },
    { name: 'dash', description: 'Debian Almquist shell (alias for sh)', category: 'Shell' },

    // Init System (6)
    { name: 'init', description: 'System init process (PID 1)', category: 'Init System' },
    { name: 'telinit', description: 'Change system runlevel', category: 'Init System' },
    { name: 'runlevel', description: 'Print current runlevel', category: 'Init System' },
    { name: 'getty', description: 'Open terminal and spawn login', category: 'Init System' },
    { name: 'sulogin', description: 'Single-user login', category: 'Init System' },
    { name: 'linuxrc', description: 'Init for initramfs (alias for init)', category: 'Init System' },

    // Text editors
    { name: 'vi', description: 'Visual text editor with modal editing', category: 'Text Processing' },
    { name: 'view', description: 'Read-only vi', category: 'Text Processing' },

    // Package Management (1)
    { name: 'apk', description: 'Alpine Linux package manager (optional)', category: 'Package Management' },

    // Archiving (14)
    { name: 'tar', description: 'Tape archive utility', category: 'Archiving' },
    { name: 'gzip', description: 'GNU zip compression', category: 'Archiving' },
    { name: 'gunzip', description: 'Decompress gzip files', category: 'Archiving' },
    { name: 'zcat', description: 'Decompress gzip to stdout', category: 'Archiving' },
    { name: 'bzip2', description: 'Burrows-Wheeler compression', category: 'Archiving' },
    { name: 'bunzip2', description: 'Decompress bzip2 files', category: 'Archiving' },
    { name: 'bzcat', description: 'Decompress bzip2 to stdout', category: 'Archiving' },
    { name: 'xz', description: 'LZMA2 compression', category: 'Archiving' },
    { name: 'unxz', description: 'Decompress xz files', category: 'Archiving' },
    { name: 'xzcat', description: 'Decompress xz to stdout', category: 'Archiving' },
    { name: 'cpio', description: 'Copy files to/from archives', category: 'Archiving' },
    { name: 'unzip', description: 'Extract ZIP archives', category: 'Archiving' },
    { name: 'compress', description: 'LZW compression (stub)', category: 'Archiving' },
    { name: 'uncompress', description: 'LZW decompression (stub)', category: 'Archiving' },

    // Misc (14)
    { name: '[', description: 'Test alias', category: 'Misc' },
    { name: 'clear', description: 'Clear screen', category: 'Misc' },
    { name: 'cmp', description: 'Compare bytes', category: 'Misc' },
    { name: 'diff', description: 'Compare files', category: 'Misc' },
    { name: 'expr', description: 'Evaluate expression', category: 'Misc' },
    { name: 'factor', description: 'Prime factors', category: 'Misc' },
    { name: 'false', description: 'Return failure', category: 'Misc' },
    { name: 'find', description: 'Search files', category: 'Misc' },
    { name: 'getconf', description: 'Get config values', category: 'Misc' },
    { name: 'mesg', description: 'Control messages', category: 'Misc' },
    { name: 'sleep', description: 'Delay execution', category: 'Misc' },
    { name: 'test', description: 'Evaluate expressions', category: 'Misc' },
    { name: 'time', description: 'Time a command', category: 'Misc' },
    { name: 'true', description: 'Return success', category: 'Misc' },
    { name: 'usleep', description: 'Microsecond sleep', category: 'Misc' },
    { name: 'which', description: 'Locate command', category: 'Misc' },
  ];

  filteredApplets = [...this.applets];

  filterApplets() {
    this.filteredApplets = this.applets.filter(applet => {
      const matchesSearch = applet.name.toLowerCase().includes(this.searchQuery.toLowerCase()) ||
                           applet.description.toLowerCase().includes(this.searchQuery.toLowerCase());
      const matchesCategory = !this.selectedCategory || applet.category === this.selectedCategory;
      return matchesSearch && matchesCategory;
    });
  }

  getUniqueCategories(): string[] {
    return [...new Set(this.filteredApplets.map(a => a.category))];
  }

  getAppletsByCategory(category: string): Applet[] {
    return this.filteredApplets.filter(a => a.category === category);
  }
}
