//
//  ContentView.swift
//  mdnotes
//
//  Created by Dylan Owen on 4/4/20.
//  Copyright © 2020 Dylan Owen. All rights reserved.
//

import SwiftUI
import WebKit

struct ContentView: View {

    let url: URL

    public init(url: URL) {
        self.url = url
    }

    @ObservedObject var webViewStore = WebViewStore()

    var body: some View {
        WebView(webView: webViewStore.webView)
            .onAppear {
                print("loading iew")
                print(self.url)
                self.webViewStore.webView.load(
                    URLRequest(url: self.url)
                )
        }
    }

//    func goBack() {
//        webViewStore.webView.goBack()
//    }
//
//    func goForward() {
//        webViewStore.webView.goForward()
//    }
}


struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView(url: URL(string: "https://google.com")!)
    }
}
