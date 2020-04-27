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
    let navigationDelegate: WebViewDelegate = WebViewDelegate()
    @ObservedObject var webViewStore: WebViewStore

    public init(url: URL) {
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
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        if let url = navigationAction.request.url {
            // TODO check our port here
            if url.host == "localhost" {
                decisionHandler(.allow)
                return
            }
            else {
                // this isn't a url for our local server so open it in our default browser
                NSWorkspace.shared.open(url)
            }
        }

        decisionHandler(.cancel)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        NotesView(url: URL(string: "https://google.com")!)
    }
}
