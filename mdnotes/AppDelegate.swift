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

    var notesWindows: [NSWindowController: UInt8] = [NSWindowController: UInt8]()

    private let runtime: MdNotesRuntime = MdNotesRuntime.shared

    func applicationWillFinishLaunching(_ notification: Notification) {
    }

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        NotificationCenter.default.addObserver(self, selector: #selector(windowDidClose(_:)), name: NSWindow.willCloseNotification, object: nil)

        openNotes()
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }

    // FOR SOME REASON AppDelegate isn't part of the first responder chain? So manually connect the Open menu item here
    @IBAction func openNotesEvent(_ sender: Any) {
        openNotes()
    }

    @objc func windowDidClose(_ notification: Notification) {
        let window = notification.object as! NSWindow

        if let notes_id = window.windowController.flatMap({ self.notesWindows.removeValue(forKey: $0) }) {
            self.runtime.closeNotes(id: notes_id)
        }
    }

    func openNotes() {
        if let path = openPrompt() {
            let window = NSWindow(
                    contentRect: NSRect(x: 0, y: 0, width: 1200, height: 800),
                    styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
                    backing: .buffered,
                    defer: false
            )
            window.title = path
            window.center()

            let controller = NSWindowController(window: window)
            controller.showWindow(self)

            // TODO this threading probably does nothing
            DispatchQueue.main.async {
                let note_id = self.runtime.openNotes(path: path)

                //print(path?.absoluteString)
                let serverPort = self.runtime.serverPort()
                let serverUrl = "http://localhost:" + String(serverPort) + "/" + String(note_id) + "/static/"

                // Create the window and set the content view.
                let contentView = NotesView(url: URL(string: serverUrl)!)
                // TODO doesn't work: contentView.navigate(url: URL(string: serverUrl)!)

                window.contentView = NSHostingView(rootView: contentView)

                self.notesWindows[controller] = note_id
            }
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
