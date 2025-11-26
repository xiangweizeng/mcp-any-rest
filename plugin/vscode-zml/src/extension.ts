import * as vscode from 'vscode';
import { ZMLFormatter } from './formatter';

export function activate(context: vscode.ExtensionContext) {
    console.log('ZML extension is now active!');
    
    // Register document formatter
    const formatter = new ZMLFormatter();
    const formatterProvider = vscode.languages.registerDocumentFormattingEditProvider('zml', {
        provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
            return formatter.formatDocument(document);
        }
    });

    context.subscriptions.push(formatterProvider);
    
    // Register range formatter
    const rangeFormatterProvider = vscode.languages.registerDocumentRangeFormattingEditProvider('zml', {
        provideDocumentRangeFormattingEdits(document: vscode.TextDocument, range: vscode.Range): vscode.TextEdit[] {
            return formatter.formatRange(document, range);
        }
    });

    context.subscriptions.push(rangeFormatterProvider);
    
    // Register on-type formatter
    const onTypeFormatterProvider = vscode.languages.registerOnTypeFormattingEditProvider('zml', {
        provideOnTypeFormattingEdits(document: vscode.TextDocument, position: vscode.Position, ch: string): vscode.TextEdit[] {
            return formatter.formatOnType(document, position, ch);
        }
    }, '\n', '}', ']', ',');

    context.subscriptions.push(onTypeFormatterProvider);
}

export function deactivate() {
    console.log('ZML extension is now deactivated!');
}