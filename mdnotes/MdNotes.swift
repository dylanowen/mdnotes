//
//  MdNotes.swift
//  mdnotes
//
//  Created by Dylan Owen on 4/5/20.
//  Copyright © 2020 Dylan Owen. All rights reserved.
//

import Foundation
import Swift

class MdNotesRuntime {

    static let shared = MdNotesRuntime()

    private let rust: OpaquePointer

    private init() {
        rust = md_notes_runtime_new()
    }
    
    func openNotes(path: String) -> UInt8 {
        let raw_path = (path as NSString).utf8String

        return md_notes_runtime_open_notes(rust, raw_path)
    }

    func closeNotes(id: UInt8) {
        md_notes_runtime_close_notes(rust, id)
    }
    
    func serverPort() -> UInt16 {
        md_notes_runtime_server_port(rust)
    }

    deinit {
        md_notes_runtime_free(rust)
    }
}
