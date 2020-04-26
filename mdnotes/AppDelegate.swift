//
//  AppDelegate.swift
//  mdnotes
//
//  Created by Dylan Owen on 4/4/20.
//  Copyright © 2020 Dylan Owen. All rights reserved.
//

import Cocoa
import SwiftUI

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate {

    var window: NSWindow!

    private let runtime: MdNotesRuntime = MdNotesRuntime.shared

    func applicationWillFinishLaunching(_ notification: Notification) {
    }

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        openNotes()
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }

    // FOR SOME REASON AppDelegate isn't part of the first responder chain? So manually connect the Open menu item here
    @IBAction func openNotesEvent(_ sender: Any) {
        openNotes()
    }

    func openNotes() {
        if let path = openPrompt() {
            let window = NSWindow(
                    contentRect: NSRect(x: 0, y: 0, width: 1200, height: 800),
                    styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
                    backing: .buffered, defer: false)
            window.center()
            //window.contentView = NSHostingView(rootView: LoadingView())
            window.title = path

            window.makeKeyAndOrderFront(nil)

            DispatchQueue.main.async {
                let note_id = self.runtime.openNotes(path: path)

                //print(path?.absoluteString)
                let serverPort = self.runtime.serverPort()
                let serverUrl = "http://localhost:" + String(serverPort) + "/" + String(note_id) + "/static/"

                // Create the window and set the content view.
                let contentView = NotesView(url: URL(string: serverUrl)!)

                contentView.navigate(url: URL(string: serverUrl)!)

                window.contentView = NSHostingView(rootView: contentView)
            }

            let windowController = NotesWindowController(window: window)
        }
    }

    func openPrompt() -> String? {
        let open = NSOpenPanel()
        open.message = "Open your notes directory"
        open.prompt = "Open"
        open.allowedFileTypes = ["none"]
        open.allowsOtherFileTypes = false
        open.canChooseFiles = false
        open.canChooseDirectories = true

        open.runModal()

        return open.url?.path
    }
}
