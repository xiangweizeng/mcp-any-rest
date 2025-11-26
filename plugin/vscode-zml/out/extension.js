"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = require("vscode");
const formatter_1 = require("./formatter");
function activate(context) {
    console.log('ZML extension is now active!');
    // Register document formatter
    const formatter = new formatter_1.ZMLFormatter();
    const formatterProvider = vscode.languages.registerDocumentFormattingEditProvider('zml', {
        provideDocumentFormattingEdits(document) {
            return formatter.formatDocument(document);
        }
    });
    context.subscriptions.push(formatterProvider);
    // Register range formatter
    const rangeFormatterProvider = vscode.languages.registerDocumentRangeFormattingEditProvider('zml', {
        provideDocumentRangeFormattingEdits(document, range) {
            return formatter.formatRange(document, range);
        }
    });
    context.subscriptions.push(rangeFormatterProvider);
    // Register on-type formatter
    const onTypeFormatterProvider = vscode.languages.registerOnTypeFormattingEditProvider('zml', {
        provideOnTypeFormattingEdits(document, position, ch) {
            return formatter.formatOnType(document, position, ch);
        }
    }, '\n', '}', ']', ',');
    context.subscriptions.push(onTypeFormatterProvider);
}
exports.activate = activate;
function deactivate() {
    console.log('ZML extension is now deactivated!');
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map