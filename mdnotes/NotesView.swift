//
//  ContentView.swift
//  mdnotes
//
//  Created by Dylan Owen on 4/4/20.
//  Copyright © 2020 Dylan Owen. All rights reserved.
//

import SwiftUI
import WebKit

struct NotesView: View {

    let initialUrl: URL
    let navigationDelegate: WebViewDelegate
    @ObservedObject var webViewStore: WebViewStore

    public init(validBaseUrl: URL, url: URL) {
        self.navigationDelegate = WebViewDelegate(validBase: validBaseUrl)

        self.initialUrl = url
        self.webViewStore = WebViewStore(navigationDelegate: self.navigationDelegate)
    }

    var body: some View {
        WebView(webView: webViewStore.webView)
                .onAppear {
                    print("loading view")
                    print(self.initialUrl)

                    self.navigate(url: self.initialUrl)
                }
    }

    func navigate(url: URL) {
        self.webViewStore.webView.load(
                URLRequest(url: self.initialUrl)
        )
    }
}

class WebViewDelegate: NSObject, WKNavigationDelegate {

    let validBase: URL

    init(validBase: URL) {
        self.validBase = validBase
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        if let url = navigationAction.request.url {
            // check scheme://host:port
            if self.validBase.scheme == url.scheme &&
                       self.validBase.host == url.host &&
                       self.validBase.port == url.port {
                decisionHandler(.allow)
                return
            } else {
                // this isn't a url for our local server so open it in our default browser
                NSWorkspace.shared.open(url)
            }
        }

        decisionHandler(.cancel)
    }
}
