import * as vscode from "vscode";
import {LanguageClient, LanguageClientOptions, ServerOptions, TransportKind,} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(): void {
    const serverPath = vscode.workspace
        .getConfiguration("fsc")
        .get<string>("server.path", "fsc-lsp");

    const serverOptions: ServerOptions = {
        command: serverPath,
        transport: TransportKind.stdio,
    };
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{language: "fsc", scheme: "file"}],
    };

    client = new LanguageClient(
        "fsc",
        "FSC Language Server",
        serverOptions,
        clientOptions,
    );
    void client.start();
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
