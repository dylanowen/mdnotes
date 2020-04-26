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

    public init(url: URL) {
        self.initialUrl = url
    }

    @ObservedObject var webViewStore = WebViewStore()

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
        NotesView(url: URL(string: "https://google.com")!)
    }
}
