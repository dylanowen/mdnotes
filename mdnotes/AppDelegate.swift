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

    func applicationWillFinishLaunching(_ notification: Notification) {
        MdNotesRuntime.shared
    }


    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Create the SwiftUI view that provides the window contents.
        let open = NSOpenPanel()
        open.message = "Open your notes directory"
        open.prompt = "Open"
        open.allowedFileTypes = ["none"]
        open.allowsOtherFileTypes = false
        open.canChooseFiles = false
        open.canChooseDirectories = true

        open.runModal()

        let path = open.url;

        let note_id = MdNotesRuntime.shared.openNotes(path: path!.path)

        //print(path?.absoluteString)
        let serverPort = MdNotesRuntime.shared.serverPort()
        let serverUrl = "http://localhost:" + String(serverPort) + "/" + String(note_id) + "/static/"

        let contentView = ContentView(url: URL(string: serverUrl)!)

        // Create the window and set the content view.
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1200, height: 800),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered, defer: false)
        window.center()
        window.setFrameAutosaveName("Main Window")
        window.contentView = NSHostingView(rootView: contentView)
        window.makeKeyAndOrderFront(nil)
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }


}
