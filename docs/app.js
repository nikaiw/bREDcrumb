// RedBreadcrumb - WASM Demo Application

let wasmModule = null;
let wasmReady = false;
const statusEl = document.getElementById('wasm-loading');

// Try to load WASM module
async function initWasm() {
    try {
        statusEl.textContent = 'Loading WebAssembly module...';
        console.log('[WASM] Starting initialization...');

        // Dynamically import the module
        const module = await import('./pkg/redbreadcrumb.js');
        console.log('[WASM] Module imported, initializing...');

        // Initialize the WASM - this loads the .wasm file
        const wasmInstance = await module.default();
        console.log('[WASM] Initialization complete, instance:', wasmInstance);

        // Store reference to the module's exported functions
        wasmModule = module;
        wasmReady = true;

        // Test that it actually works
        try {
            const testStr = module.generate_string(8, 'TEST');
            console.log('[WASM] Test call successful:', testStr);
        } catch (testErr) {
            console.error('[WASM] Test call failed:', testErr);
            throw testErr;
        }

        statusEl.textContent = 'WebAssembly loaded successfully!';
        statusEl.className = 'wasm-status loaded';

        console.log('[WASM] Available functions:', Object.keys(module).filter(k => typeof module[k] === 'function'));
    } catch (e) {
        console.error('[WASM] Initialization failed:', e);
        statusEl.textContent = 'Using JavaScript fallback (WASM unavailable)';
        statusEl.className = 'wasm-status';
        wasmReady = false;
        wasmModule = null;
    }
}

// Helper to safely call WASM functions
function callWasm(fn, ...args) {
    if (!wasmReady || !wasmModule) {
        console.log(`[WASM] Not ready, can't call ${fn}`);
        return null;
    }
    if (typeof wasmModule[fn] !== 'function') {
        console.error(`[WASM] Function ${fn} not found`);
        return null;
    }
    try {
        console.log(`[WASM] Calling ${fn} with args:`, args);
        const result = wasmModule[fn](...args);
        console.log(`[WASM] ${fn} returned:`, result);
        return result;
    } catch (e) {
        console.error(`[WASM] ${fn} error:`, e);
        throw e;
    }
}

// Fallback implementations (when WASM is not available)
const fallback = {
    generateString(length, prefix) {
        const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        let result = prefix;
        for (let i = prefix.length; i < length; i++) {
            result += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return result;
    },

    generateCode(str, lang) {
        const templates = {
            c: `#include <stdio.h>

/* Tracking string - DO NOT REMOVE */
/* This string is used for binary attribution/tracking */

static volatile const char TRACKING_STRING[] = "${str}";

/* Call this function once at startup to ensure the string is kept */
__attribute__((constructor, used))
static void _tracking_init(void) {
    volatile const char* p = TRACKING_STRING;
    (void)p;
}`,
            cpp: `#include <cstdio>

// Tracking string - DO NOT REMOVE
// This string is used for binary attribution/tracking

static volatile const char TRACKING_STRING[] = "${str}";

// Call this function once at startup to ensure the string is kept
__attribute__((constructor, used))
static void _tracking_init() {
    volatile const char* p = TRACKING_STRING;
    (void)p;
}`,
            rust: `// Tracking string - DO NOT REMOVE
// This string is used for binary attribution/tracking

#[used]
#[no_mangle]
static TRACKING_STRING: &[u8] = b"${str}";

// Ensure the string is not optimized away
#[inline(never)]
fn _tracking_init() {
    let _ = unsafe { std::ptr::read_volatile(&TRACKING_STRING) };
}`,
            go: `package main

// Tracking string - DO NOT REMOVE
// This string is used for binary attribution/tracking

var trackingString = "${str}"

// getTrackingString prevents the compiler from optimizing away the string
//
//go:noinline
func getTrackingString() string {
	return trackingString
}

// init ensures the string is kept in the binary
func init() {
	_ = getTrackingString()
}`,
            csharp: `// Tracking string - DO NOT REMOVE
// This string is used for binary attribution/tracking

using System.Runtime.CompilerServices;

public static class TrackingString
{
    public static readonly string Value = "${str}";

    // Static constructor ensures the string is kept
    [MethodImpl(MethodImplOptions.NoInlining)]
    static TrackingString()
    {
        _ = Value.Length;
    }
}`,
            java: `// Tracking string - DO NOT REMOVE
// This string is used for binary attribution/tracking

public class TrackingString {
    public static final String VALUE = "${str}";

    // Static block ensures the string is kept in the class file
    static {
        @SuppressWarnings("unused")
        int len = VALUE.length();
    }
}`
        };
        return templates[lang] || templates.c;
    },

    generateYara(str, ascii, wide) {
        const hexPattern = Array.from(str).map(c =>
            c.charCodeAt(0).toString(16).padStart(2, '0').toUpperCase()
        ).join(' ');

        const modifiers = [];
        if (ascii) modifiers.push('ascii');
        if (wide) modifiers.push('wide');
        const modStr = modifiers.length ? ' ' + modifiers.join(' ') : '';

        const ruleName = `tracking_string_${str.substring(0, 8).replace(/[^a-zA-Z0-9]/g, '_')}`;

        return `rule ${ruleName} {
    meta:
        description = "Detects tracking string: ${str}"
        author = "redbreadcrumb"
        date = "${new Date().toISOString().split('T')[0]}"

    strings:
        $tracking_string = "${str}"${modStr}
        $tracking_hex = { ${hexPattern} }

    condition:
        any of them
}`;
    }
};

// Event handlers
document.getElementById('btn-generate').addEventListener('click', () => {
    const length = parseInt(document.getElementById('gen-length').value) || 16;
    const prefix = document.getElementById('gen-prefix').value || 'RT';

    let result;
    try {
        if (wasmReady) {
            result = callWasm('generate_string', length, prefix);
        }
        if (!result) {
            result = fallback.generateString(length, prefix);
        }
    } catch (e) {
        console.error('Generate error:', e);
        result = fallback.generateString(length, prefix);
    }

    document.getElementById('gen-output').textContent = result;

    // Auto-populate other inputs
    document.getElementById('code-string').value = result;
    document.getElementById('yara-string').value = result;
    document.getElementById('patch-string').value = result;
});

document.getElementById('btn-code').addEventListener('click', () => {
    const str = document.getElementById('code-string').value;
    if (!str) {
        document.getElementById('code-output').textContent = 'Please enter a tracking string';
        return;
    }

    const lang = document.getElementById('code-lang').value;

    let result;
    try {
        if (wasmReady) {
            // WASM expects: (tracking_string, language)
            result = callWasm('generate_code', str, lang);
        }
    } catch (e) {
        console.error('WASM generate_code error, using fallback:', e);
        result = null;
    }

    if (!result) {
        result = fallback.generateCode(str, lang);
    }

    document.getElementById('code-output').textContent = result;
});

document.getElementById('btn-yara').addEventListener('click', () => {
    const str = document.getElementById('yara-string').value;
    if (!str) {
        document.getElementById('yara-output').textContent = 'Please enter a tracking string';
        return;
    }

    const ascii = document.getElementById('yara-ascii').checked;
    const wide = document.getElementById('yara-wide').checked;

    let result;
    try {
        if (wasmReady) {
            // WASM expects: (tracking_string, rule_name, ascii, wide)
            result = callWasm('generate_yara', str, null, ascii, wide);
        }
    } catch (e) {
        console.error('WASM generate_yara error, using fallback:', e);
        result = null;
    }

    if (!result) {
        result = fallback.generateYara(str, ascii, wide);
    }

    document.getElementById('yara-output').textContent = result;
});

document.getElementById('btn-patch').addEventListener('click', async () => {
    const fileInput = document.getElementById('patch-file');
    const str = document.getElementById('patch-string').value;
    const strategy = document.getElementById('patch-strategy').value;
    const outputEl = document.getElementById('patch-output');

    if (!fileInput.files.length) {
        outputEl.textContent = 'Please select a binary file';
        return;
    }

    if (!str) {
        outputEl.textContent = 'Please enter a tracking string';
        return;
    }

    const file = fileInput.files[0];
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);

    outputEl.textContent = `Processing ${file.name} (${data.length} bytes)...`;

    if (!wasmReady) {
        outputEl.innerHTML = `
<span style="color: #f85149;">WASM module required for binary patching</span>

To use this feature, build and deploy the WASM module:
<code>wasm-pack build --target web --out-dir docs/pkg</code>`;
        return;
    }

    try {
        const result = callWasm('patch_binary', data, str, strategy);

        if (!result) {
            throw new Error('Patching failed - no result returned');
        }

        // Create download link
        const blob = new Blob([result], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const filename = file.name.replace(/(\.[^.]+)?$/, '_patched$1');

        outputEl.innerHTML = `
<span style="color: #238636;">Successfully patched binary!</span>
  Original size: ${data.length} bytes
  Patched size: ${result.length} bytes
  Strategy: ${strategy}

<a href="${url}" download="${filename}" class="btn btn-primary" style="display: inline-block; margin-top: 10px;">Download Patched Binary</a>`;
    } catch (e) {
        outputEl.innerHTML = `<span style="color: #f85149;">Error: ${e.message || e}</span>`;
    }
});

// Initialize
initWasm();
