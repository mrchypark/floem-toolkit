#!/usr/bin/env swift

import Carbon
import Foundation

enum HelperError: Error, CustomStringConvertible {
    case usage
    case noCurrentSource
    case noSavedSource(URL)
    case noKorean2SetSource
    case sourceSelectionFailed(String, OSStatus)
    case unableToReadSources

    var description: String {
        switch self {
        case .usage:
            return "usage: input_source_helper.swift set-korean | restore"
        case .noCurrentSource:
            return "could not read the current input source"
        case .noSavedSource(let url):
            return "no saved input source at \(url.path)"
        case .noKorean2SetSource:
            return "could not find a Korean 2-set input source"
        case .sourceSelectionFailed(let id, let status):
            return "failed to select input source \(id): \(status)"
        case .unableToReadSources:
            return "could not enumerate input sources"
        }
    }
}

struct InputSource {
    let raw: TISInputSource
    let id: String
    let name: String

    init?(raw: TISInputSource) {
        self.raw = raw

        guard let idValue = TISGetInputSourceProperty(raw, kTISPropertyInputSourceID),
              let nameValue = TISGetInputSourceProperty(raw, kTISPropertyLocalizedName)
        else {
            return nil
        }

        let idString = Unmanaged<CFString>.fromOpaque(UnsafeRawPointer(idValue)).takeUnretainedValue()
        let nameString = Unmanaged<CFString>.fromOpaque(UnsafeRawPointer(nameValue)).takeUnretainedValue()
        self.id = idString as String
        self.name = nameString as String
    }

    var isKorean2SetCandidate: Bool {
        let lowerID = id.lowercased()
        let lowerName = name.lowercased()
        if lowerID == "com.apple.inputmethod.korean.2setkorean" {
            return true
        }
        if lowerID.contains("korean") && lowerID.contains("2set") {
            return true
        }
        if lowerName.contains("korean") && (lowerName.contains("2-set") || lowerName.contains("2 set") || lowerName.contains("2set")) {
            return true
        }
        return false
    }
}

let stateFileURL = URL(
    fileURLWithPath: ProcessInfo.processInfo.environment["IME_INPUT_SOURCE_STATE_FILE"]
        ?? (FileManager.default.currentDirectoryPath + "/artifacts/ime/_state/input_source_previous.txt")
)

func ensureStateDirectoryExists() throws {
    let directory = stateFileURL.deletingLastPathComponent()
    try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
}

func currentSource() throws -> InputSource {
    guard let unmanaged = TISCopyCurrentKeyboardInputSource() else {
        throw HelperError.noCurrentSource
    }
    let raw = unmanaged.takeRetainedValue()
    guard let source = InputSource(raw: raw) else {
        throw HelperError.noCurrentSource
    }
    return source
}

func allSources() throws -> [InputSource] {
    let rawList = TISCreateInputSourceList(nil, false)
    guard let unmanagedList = rawList else {
        throw HelperError.unableToReadSources
    }
    let list = unmanagedList.takeRetainedValue() as NSArray
    return list.compactMap { element in
        let raw = element as! TISInputSource
        return InputSource(raw: raw)
    }
}

func selectSource(withID id: String) throws {
    let sources = try allSources()
    guard let source = sources.first(where: { $0.id == id }) else {
        throw HelperError.sourceSelectionFailed(id, -1)
    }
    let status = TISSelectInputSource(source.raw)
    guard status == noErr else {
        throw HelperError.sourceSelectionFailed(id, status)
    }
}

func korean2SetSource() throws -> InputSource {
    let sources = try allSources()
    if let exact = sources.first(where: { $0.id == "com.apple.inputmethod.Korean.2SetKorean" }) {
        return exact
    }
    if let candidate = sources.first(where: { $0.isKorean2SetCandidate }) {
        return candidate
    }
    throw HelperError.noKorean2SetSource
}

func saveCurrentSource(_ source: InputSource) throws {
    try ensureStateDirectoryExists()
    try source.id.write(to: stateFileURL, atomically: true, encoding: .utf8)
}

func loadSavedSourceID() throws -> String {
    let data = try String(contentsOf: stateFileURL, encoding: .utf8)
    let trimmed = data.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        throw HelperError.noSavedSource(stateFileURL)
    }
    return trimmed
}

func setKorean2Set() throws {
    let source = try currentSource()
    try saveCurrentSource(source)
    let target = try korean2SetSource()
    if source.id != target.id {
        let status = TISSelectInputSource(target.raw)
        guard status == noErr else {
            throw HelperError.sourceSelectionFailed(target.id, status)
        }
    }
}

func restorePreviousSource() throws {
    let savedID = try loadSavedSourceID()
    try selectSource(withID: savedID)
}

do {
    let mode = CommandLine.arguments.dropFirst().first
    guard let command = mode else {
        throw HelperError.usage
    }

    switch command {
    case "set-korean":
        try setKorean2Set()
    case "restore":
        try restorePreviousSource()
    default:
        throw HelperError.usage
    }
} catch {
    fputs("\(error)\n", stderr)
    if let helperError = error as? HelperError, case .usage = helperError {
        exit(2)
    }
    exit(1)
}
