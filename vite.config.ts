import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import {NodeGlobalsPolyfillPlugin} from '@esbuild-plugins/node-globals-polyfill'
import path from 'path';
import rollupNodePolyFill from 'rollup-plugin-polyfill-node'
import { nodePolyfills } from 'vite-plugin-node-polyfills'
// https://vitejs.dev/config/
export default defineConfig({
   build: {
     lib: {
       entry: path.resolve(__dirname, 'src/cip30/InjectScript.ts'),
       name: 'InjectScript',
       fileName: (format) => `InjectScript.${format}.js`,
       formats: ['es'],
     },
     
     rollupOptions: {
       output: {
         inlineDynamicImports: true,  // Ensure dynamic imports are inlined
       },
       /* plugins:[
        wasm(), 
        topLevelAwait(),
        nodePolyfills(),
        NodeGlobalsPolyfillPlugin({
          buffer: true
        }),
      ] */
       
     },
   },
  /*  build:{
    rollupOptions: {
      plugins: [
        rollupNodePolyFill()
      ]
    }
   }, */
  resolve: {
    alias: {
      '@libs/libs/cardano_multiplatform_lib': path.resolve(__dirname, 'src/cml/libs/cardano_multiplatform_lib'),
      '@libs/cardano_message_signing': path.resolve(__dirname, 'src/cml/libs/cardano_message_signing'),
    }
  },
  define: {
    global: {},
  },
  assetsInclude: ['**/*.wasm'],
  plugins: [
    nodePolyfills(),
    NodeGlobalsPolyfillPlugin({
      buffer: true
    }),
    react(),
    wasm(),
    topLevelAwait(),
    
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
