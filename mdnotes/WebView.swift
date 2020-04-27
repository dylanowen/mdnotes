import SwiftUI
import Combine
import WebKit

/**
 Borrowed from https://github.com/kylehickinson/SwiftUI-WebView
 */
public class WebViewStore: ObservableObject {
    @Published public var webView: WKWebView {
        didSet {
            setupObservers()
        }
    }

    public init(navigationDelegate: WKNavigationDelegate? = nil) {
        self.webView = WKWebView()
        self.webView.allowsBackForwardNavigationGestures = true
        self.webView.navigationDelegate = navigationDelegate

        setupObservers()
    }

    private func setupObservers() {
        func subscriber<Value>(for keyPath: KeyPath<WKWebView, Value>) -> NSKeyValueObservation {
            webView.observe(keyPath, options: [.prior]) { _, change in
                if change.isPrior {
                    self.objectWillChange.send()
                }
            }
        }

        // Setup observers for all KVO compliant properties
        observers = [
            subscriber(for: \.title),
            subscriber(for: \.url),
            subscriber(for: \.isLoading),
            subscriber(for: \.estimatedProgress),
            subscriber(for: \.hasOnlySecureContent),
            subscriber(for: \.serverTrust),
            subscriber(for: \.canGoBack),
            subscriber(for: \.canGoForward)
        ]
    }

    private var observers: [NSKeyValueObservation] = []

    deinit {
        observers.forEach {
            // Not even sure if this is required?
            // Probably wont be needed in future betas?
            $0.invalidate()
        }
    }
}

/// A container for using a WKWebView in SwiftUI
public struct WebView: View, NSViewRepresentable {

    /// The WKWebView to display
    public let webView: WKWebView

    public typealias NSViewType = NSViewContainerView<WKWebView>

    public init(webView: WKWebView) {
        self.webView = webView
    }

    public func makeNSView(context: Context) -> NSViewContainerView<WKWebView> {
        return NSViewContainerView()
    }

    public func updateNSView(_ nsView: WebView.NSViewType, context: NSViewRepresentableContext<WebView>) {
        // If its the same content view we don't need to update.
        if nsView.contentView !== webView {
            nsView.contentView = webView
        }
    }
}

/// A UIView which simply adds some view to its view hierarchy
public class NSViewContainerView<ContentView: NSView>: NSView {
    var contentView: ContentView? {
        willSet {
            contentView?.removeFromSuperview()
        }
        didSet {
            if let contentView = contentView {
                addSubview(contentView)
                contentView.translatesAutoresizingMaskIntoConstraints = false
                NSLayoutConstraint.activate([
                    contentView.leadingAnchor.constraint(equalTo: leadingAnchor),
                    contentView.trailingAnchor.constraint(equalTo: trailingAnchor),
                    contentView.topAnchor.constraint(equalTo: topAnchor),
                    contentView.bottomAnchor.constraint(equalTo: bottomAnchor)
                ])
            }
        }
    }
}
