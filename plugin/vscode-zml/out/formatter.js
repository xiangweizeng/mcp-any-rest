"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ZMLFormatter = void 0;
const vscode = require("vscode");
class ZMLFormatter {
    constructor() {
        this.config = vscode.workspace.getConfiguration('zml.format');
    }
    formatDocument(document) {
        if (!this.config.get('enable', true)) {
            return [];
        }
        const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
        return this.formatRange(document, fullRange);
    }
    formatRange(document, range) {
        if (!this.config.get('enable', true)) {
            return [];
        }
        const text = document.getText(range);
        const formattedText = this.formatText(text);
        return [vscode.TextEdit.replace(range, formattedText)];
    }
    formatOnType(document, position, ch) {
        if (!this.config.get('enable', true)) {
            return [];
        }
        // Simple on-type formatting for common cases
        const line = document.lineAt(position.line);
        const lineText = line.text;
        if (ch === '\n') {
            return this.handleNewLine(document, position, lineText);
        }
        if (ch === '}' || ch === ']') {
            return this.handleClosingBrace(document, position, lineText, ch);
        }
        if (ch === ',') {
            return this.handleComma(document, position, lineText);
        }
        return [];
    }
    formatText(text) {
        const indentSize = this.config.get('indentSize', 2);
        const maxLineLength = this.config.get('maxLineLength', 120);
        const lines = text.split('\n');
        let formattedLines = [];
        let indentLevel = 0;
        let inBlockComment = false;
        for (let i = 0; i < lines.length; i++) {
            let line = lines[i].trimEnd();
            // Handle block comments
            if (inBlockComment) {
                if (line.includes('*/')) {
                    inBlockComment = false;
                }
                formattedLines.push(this.indentLine(line, indentLevel, indentSize));
                continue;
            }
            if (line.includes('/*')) {
                inBlockComment = true;
            }
            // Skip empty lines
            if (line.trim() === '') {
                formattedLines.push('');
                continue;
            }
            // Handle line comments
            if (line.trim().startsWith('//')) {
                formattedLines.push(this.indentLine(line, indentLevel, indentSize));
                continue;
            }
            // Adjust indent level based on braces
            const openBraces = (line.match(/\{/g) || []).length;
            const closeBraces = (line.match(/\}/g) || []).length;
            // Apply current indent level
            const currentIndentLevel = Math.max(0, indentLevel);
            // Format the line
            let formattedLine = this.indentLine(line, currentIndentLevel, indentSize);
            // Handle long lines
            if (formattedLine.length > maxLineLength) {
                formattedLine = this.breakLongLine(formattedLine, maxLineLength, currentIndentLevel, indentSize);
            }
            formattedLines.push(formattedLine);
            // Update indent level for next line
            indentLevel += openBraces - closeBraces;
        }
        return formattedLines.join('\n');
    }
    indentLine(line, indentLevel, indentSize) {
        const indent = ' '.repeat(indentLevel * indentSize);
        return indent + line.trimStart();
    }
    breakLongLine(line, maxLength, indentLevel, indentSize) {
        // Simple line breaking for common ZML patterns
        const indent = ' '.repeat(indentLevel * indentSize);
        const nextIndent = ' '.repeat((indentLevel + 1) * indentSize);
        // Break at commas in arrays/objects
        if (line.includes(',') && line.length > maxLength) {
            const parts = line.split(',');
            let result = parts[0];
            for (let i = 1; i < parts.length; i++) {
                const currentLine = result + ',' + parts[i];
                if (currentLine.length > maxLength) {
                    result += ',';
                    result += '\n' + nextIndent + parts[i].trimStart();
                }
                else {
                    result += ',' + parts[i];
                }
            }
            return result;
        }
        // Break long method declarations
        if (line.includes('(') && line.includes(')') && line.length > maxLength) {
            const beforeParen = line.substring(0, line.indexOf('(') + 1);
            const afterParen = line.substring(line.indexOf('(') + 1);
            if (beforeParen.length + 10 < maxLength) {
                return beforeParen + '\n' + nextIndent + afterParen;
            }
        }
        return line; // Return original if we can't break it nicely
    }
    handleNewLine(document, position, lineText) {
        const indentSize = this.config.get('indentSize', 2);
        // Calculate indent level for the new line
        const previousLine = document.lineAt(position.line - 1);
        const prevText = previousLine.text;
        let indentLevel = this.getIndentLevel(prevText, indentSize);
        // Increase indent if previous line ends with {
        if (prevText.trim().endsWith('{')) {
            indentLevel++;
        }
        // Decrease indent if current line starts with }
        if (lineText.trim().startsWith('}')) {
            indentLevel = Math.max(0, indentLevel - 1);
        }
        const indent = ' '.repeat(indentLevel * indentSize);
        const editPosition = new vscode.Position(position.line, 0);
        return [vscode.TextEdit.insert(editPosition, indent)];
    }
    handleClosingBrace(document, position, lineText, ch) {
        const indentSize = this.config.get('indentSize', 2);
        // Get the text before the closing brace
        const textBeforeBrace = lineText.substring(0, position.character);
        // If the line only contains whitespace and the brace, adjust indentation
        if (textBeforeBrace.trim() === '') {
            const previousLine = document.lineAt(position.line - 1);
            const prevIndentLevel = this.getIndentLevel(previousLine.text, indentSize);
            const currentIndentLevel = Math.max(0, prevIndentLevel - 1);
            const correctIndent = ' '.repeat(currentIndentLevel * indentSize);
            const range = new vscode.Range(new vscode.Position(position.line, 0), new vscode.Position(position.line, textBeforeBrace.length));
            return [vscode.TextEdit.replace(range, correctIndent)];
        }
        return [];
    }
    handleComma(document, position, lineText) {
        // Ensure space after comma
        const nextChar = document.getText(new vscode.Range(position, new vscode.Position(position.line, position.character + 1)));
        if (nextChar !== ' ' && nextChar !== '\n' && nextChar !== '') {
            return [vscode.TextEdit.insert(position, ' ')];
        }
        return [];
    }
    getIndentLevel(line, indentSize) {
        const leadingSpaces = line.match(/^ */)?.[0].length || 0;
        return Math.floor(leadingSpaces / indentSize);
    }
}
exports.ZMLFormatter = ZMLFormatter;
//# sourceMappingURL=formatter.js.map