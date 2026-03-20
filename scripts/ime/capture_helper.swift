import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

enum CaptureError: Error, CustomStringConvertible {
    case usage
    case invalidWindowId(String)
    case invalidNumber(String)
    case failedWindowCapture(CGWindowID)
    case failedRegionCapture(CGRect)
    case failedDestination(URL)
    case failedFinalize(URL)

    var description: String {
        switch self {
        case .usage:
            return "usage: capture_helper.swift <output_file> <window_id> <x> <y> <w> <h>"
        case let .invalidWindowId(value):
            return "invalid window id: \(value)"
        case let .invalidNumber(value):
            return "invalid numeric argument: \(value)"
        case let .failedWindowCapture(windowID):
            return "failed to capture window \(windowID)"
        case let .failedRegionCapture(rect):
            return "failed to capture region \(rect)"
        case let .failedDestination(url):
            return "failed to create image destination for \(url.path)"
        case let .failedFinalize(url):
            return "failed to write image to \(url.path)"
        }
    }
}

func parseInt(_ value: String) throws -> Int {
    guard let parsed = Int(value) else {
        throw CaptureError.invalidNumber(value)
    }
    return parsed
}

func writeImage(_ image: CGImage, to outputURL: URL) throws {
    guard let destination = CGImageDestinationCreateWithURL(
        outputURL as CFURL,
        UTType.png.identifier as CFString,
        1,
        nil
    ) else {
        throw CaptureError.failedDestination(outputURL)
    }
    CGImageDestinationAddImage(destination, image, nil)
    guard CGImageDestinationFinalize(destination) else {
        throw CaptureError.failedFinalize(outputURL)
    }
}

let arguments = CommandLine.arguments
guard arguments.count == 7 else {
    fputs("\(CaptureError.usage)\n", stderr)
    exit(2)
}

let outputURL = URL(fileURLWithPath: arguments[1])
let windowIDValue = arguments[2]
let windowID: CGWindowID
if let parsed = UInt32(windowIDValue) {
    windowID = CGWindowID(parsed)
} else {
    fputs("\(CaptureError.invalidWindowId(windowIDValue))\n", stderr)
    exit(2)
}

do {
    let x = try parseInt(arguments[3])
    let y = try parseInt(arguments[4])
    let w = try parseInt(arguments[5])
    let h = try parseInt(arguments[6])

    try FileManager.default.createDirectory(
        at: outputURL.deletingLastPathComponent(),
        withIntermediateDirectories: true
    )

    if windowID != 0 {
        if let image = CGWindowListCreateImage(
            .null,
            .optionIncludingWindow,
            windowID,
            [.bestResolution, .boundsIgnoreFraming]
        ) {
            try writeImage(image, to: outputURL)
            print(outputURL.path)
            exit(0)
        }
    }

    let rect = CGRect(x: x, y: y, width: w, height: h)
    guard let image = CGWindowListCreateImage(
        rect,
        .optionOnScreenOnly,
        kCGNullWindowID,
        [.bestResolution]
    ) else {
        throw CaptureError.failedRegionCapture(rect)
    }

    try writeImage(image, to: outputURL)
    print(outputURL.path)
} catch {
    fputs("\(error)\n", stderr)
    exit(1)
}
