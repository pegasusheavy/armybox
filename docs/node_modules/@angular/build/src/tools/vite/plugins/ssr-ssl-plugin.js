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
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAngularServerSideSSLPlugin = createAngularServerSideSSLPlugin;
const promises_1 = require("node:fs/promises");
const node_tls_1 = require("node:tls");
function createAngularServerSideSSLPlugin() {
    return {
        name: 'angular-ssr-ssl-plugin',
        apply: 'serve',
        async configureServer({ config, httpServer }) {
            const { ssr, server: { https }, } = config;
            if (!ssr || !https?.cert) {
                return;
            }
            if (httpServer && 'ALPNProtocols' in httpServer) {
                // Force Vite to use HTTP/1.1 when SSR and SSL are enabled.
                // This is required because the Express server used for SSR does not support HTTP/2.
                // See: https://github.com/vitejs/vite/blob/46d3077f2b63771cc50230bc907c48f5773c00fb/packages/vite/src/node/http.ts#L126
                // We directly set the `ALPNProtocols` on the HTTP server to override the default behavior.
                // Passing `ALPNProtocols` in the TLS options would cause Node.js to automatically include `h2`.
                // Additionally, using `ALPNCallback` is not an option as it is mutually exclusive with `ALPNProtocols`.
                // See: https://github.com/nodejs/node/blob/b8b4350ed3b73d225eb9e628d69151df56eaf298/lib/internal/http2/core.js#L3351
                httpServer.ALPNProtocols = ['http/1.1'];
            }
            const { cert } = https;
            const additionalCerts = Array.isArray(cert) ? cert : [cert];
            // TODO(alanagius): Remove the `if` check once we only support Node.js 22.18.0+ and 24.5.0+.
            if (node_tls_1.getCACertificates && node_tls_1.setDefaultCACertificates) {
                const currentCerts = (0, node_tls_1.getCACertificates)('default');
                (0, node_tls_1.setDefaultCACertificates)([...currentCerts, ...additionalCerts]);
                return;
            }
            // TODO(alanagius): Remove the below and `undici` dependency once we only support Node.js 22.18.0+ and 24.5.0+.
            const { getGlobalDispatcher, setGlobalDispatcher, Agent } = await Promise.resolve().then(() => __importStar(require('undici')));
            const originalDispatcher = getGlobalDispatcher();
            const ca = [...node_tls_1.rootCertificates, ...additionalCerts];
            const extraNodeCerts = process.env['NODE_EXTRA_CA_CERTS'];
            if (extraNodeCerts) {
                ca.push(await (0, promises_1.readFile)(extraNodeCerts));
            }
            setGlobalDispatcher(new Agent({
                connect: {
                    ca,
                },
            }));
            httpServer?.on('close', () => {
                setGlobalDispatcher(originalDispatcher);
            });
        },
    };
}
//# sourceMappingURL=ssr-ssl-plugin.js.map