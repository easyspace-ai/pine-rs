// pine-tv editor module
// Monaco editor integration with Pine Script syntax highlighting

let editor = null;
let checkDebounceTimer = null;

const DEFAULT_CODE = `//@version=6
indicator("SMA 20", overlay=true)
s = ta.sma(close, 20)
plot(s, title="SMA", color=#2196F3, linewidth=2)
`;

// Initialize
export function init() {
    loadMonaco().then(() => {
        initEditor();
        initEventListeners();
        console.log('[editor] initialized');
    });
}

// Load Monaco editor from CDN
async function loadMonaco() {
    if (window.monaco) {
        return;
    }

    return new Promise((resolve, reject) => {
        const loader = document.createElement('script');
        loader.src = 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs/loader.js';
        loader.onload = () => {
            require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' } });
            require(['vs/editor/editor.main'], () => {
                registerPineLanguage();
                resolve();
            }, reject);
        };
        loader.onerror = reject;
        document.head.appendChild(loader);
    });
}

// Register Pine Script language with Monaco
function registerPineLanguage() {
    if (!window.monaco) return;

    monaco.languages.register({ id: 'pine' });

    monaco.languages.setMonarchTokensProvider('pine', {
        keywords: [
            'if', 'else', 'for', 'while', 'switch', 'case', 'default',
            'var', 'varip', 'let', 'const',
            'function', 'method', 'type', 'struct',
            'import', 'export', 'library',
            'indicator', 'strategy', 'input', 'plot', 'hline', 'bgcolor',
            'true', 'false', 'na', 'null',
            'int', 'float', 'bool', 'string', 'color', 'series',
            'array', 'matrix', 'map', 'table', 'box', 'label', 'line',
        ],

        typeKeywords: ['int', 'float', 'bool', 'string', 'color', 'series', 'array', 'matrix', 'map'],

        tokenizer: {
            root: [
                // Comments
                [/\/\/.*/, 'comment'],
                [/\/\*[\s\S]*?\*\//, 'comment'],

                // Strings
                [/"[^"\\]*(\\.[^"\\]*)*"/, 'string'],
                [/'[^'\\]*(\\.[^'\\]*)*'/, 'string'],

                // Colors
                [/#[0-9a-fA-F]{8}/, 'number'],
                [/#[0-9a-fA-F]{6}/, 'number'],

                // Numbers
                [/\d+\.\d+([eE][+\-]?\d+)?/, 'number.float'],
                [/\d+([eE][+\-]?\d+)?/, 'number'],

                // Identifiers
                [/[a-zA-Z_]\w*/, {
                    cases: {
                        '@keywords': 'keyword',
                        '@typeKeywords': 'type',
                        '@default': 'identifier',
                    }
                }],

                // Whitespace
                [/[ \t\r\n]+/, 'white'],

                // Operators
                [/[=+\-*/%^&|!<>?:]+/, 'operator'],

                // Brackets
                [/[()\[\]{}]/, '@brackets'],
            ],
        },
    });

    monaco.editor.defineTheme('pine-dark', {
        base: 'vs-dark',
        inherit: true,
        rules: [
            { token: 'comment', foreground: '6A9955' },
            { token: 'keyword', foreground: '569CD6' },
            { token: 'type', foreground: '4EC9B0' },
            { token: 'string', foreground: 'CE9178' },
            { token: 'number', foreground: 'B5CEA8' },
            { token: 'number.float', foreground: 'B5CEA8' },
            { token: 'operator', foreground: 'D4D4D4' },
            { token: 'identifier', foreground: '9CDCFE' },
        ],
        colors: {
            'editor.background': '#1e1e1e',
            'editor.lineHighlightBackground': '#2a2a2a',
        },
    });
}

// Create editor instance
function initEditor() {
    const container = document.getElementById('editor-container');
    if (!container) return;

    editor = monaco.editor.create(container, {
        value: DEFAULT_CODE,
        language: 'pine',
        theme: 'pine-dark',
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: 13,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        renderLineHighlight: 'all',
        wordWrap: 'on',
        tabSize: 4,
        insertSpaces: true,
    });

    // Notify app of initial code
    window.dispatchEvent(new CustomEvent('pine:code-change', {
        detail: { code: DEFAULT_CODE }
    }));

    // Listen for changes
    editor.onDidChangeModelContent(() => {
        const code = editor.getValue();
        window.dispatchEvent(new CustomEvent('pine:code-change', {
            detail: { code }
        }));

        // Debounce check
        clearTimeout(checkDebounceTimer);
        checkDebounceTimer = setTimeout(() => checkCode(code), 300);
    });
}

// Check code for errors
async function checkCode(code) {
    try {
        const response = await fetch('/api/check', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ code }),
        });

        const result = await response.json();
        clearMarkers();

        if (!result.ok && result.errors) {
            setMarkers(result.errors);
        }
    } catch (e) {
        console.warn('[editor] Check failed:', e);
    }
}

// Set error markers
function setMarkers(errors) {
    if (!editor || !window.monaco) return;

    const model = editor.getModel();
    if (!model) return;

    const markers = errors.map(err => ({
        startLineNumber: err.line || 1,
        startColumn: err.col || 1,
        endLineNumber: err.line || 1,
        endColumn: (err.end_col || err.col || 1) + 1,
        message: err.msg,
        severity: monaco.MarkerSeverity.Error,
    }));

    monaco.editor.setModelMarkers(model, 'pine-tv', markers);
}

// Clear markers
function clearMarkers() {
    if (!editor || !window.monaco) return;

    const model = editor.getModel();
    if (model) {
        monaco.editor.setModelMarkers(model, 'pine-tv', []);
    }
}

// Event listeners
function initEventListeners() {
    window.addEventListener('pine:editor-set-code', (e) => {
        if (editor) {
            editor.setValue(e.detail.code);
        }
    });

    window.addEventListener('pine:error', (e) => {
        const errors = e.detail.errors || [];
        setMarkers(errors);
    });
}

// Initialize on load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
