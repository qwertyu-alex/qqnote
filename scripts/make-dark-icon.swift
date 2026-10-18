// Makes the dark-mode Dock icon from the light app icon: light lines on a dark tile.
//
//   swift scripts/make-dark-icon.swift app-icon.png src-tauri/icons/icon-dark.png
//
// Rerun it whenever app-icon.png changes. Expects a black-on-white drawing on a rounded tile
// with a soft shadow around it.
import AppKit

let args = CommandLine.arguments
guard args.count == 3,
    let data = FileManager.default.contents(atPath: args[1]),
    let source = NSBitmapImageRep(data: data),
    source.bitsPerSample == 8, source.samplesPerPixel == 4,
    source.bitmapFormat.contains(.alphaNonpremultiplied), !source.bitmapFormat.contains(.alphaFirst)
else {
    print("usage: swift make-dark-icon.swift <light RGBA png> <output png>")
    exit(1)
}

let width = source.pixelsWide, height = source.pixelsHigh
let input = source.bitmapData!, inputRow = source.bytesPerRow
let full = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height, bitsPerSample: 8,
    samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
    bitmapFormat: .alphaNonpremultiplied, bytesPerRow: width * 4, bitsPerPixel: 32)!
let output = full.bitmapData!

func alpha(_ x: Int, _ y: Int) -> Double { Double(input[y * inputRow + x * 4 + 3]) }
func smoothstep(_ a: Double, _ b: Double, _ x: Double) -> Double {
    let t = min(max((x - a) / (b - a), 0), 1)
    return t * t * (3 - 2 * t)
}

// The tile's top and bottom rows, for the background gradient.
var top = height, bottom = 0
for y in 0..<height {
    for x in stride(from: 0, to: width, by: 4) where alpha(x, y) > 200 {
        top = min(top, y)
        bottom = max(bottom, y)
        break
    }
}

// Blurred alpha dips near the tile's edge, which gets a faint rim so it shows on a dark Dock.
let radius = Int(Double(width) * 0.006)
var rows = [Double](repeating: 0, count: width * height)
var blurred = [Double](repeating: 0, count: width * height)
for y in 0..<height {
    for x in 0..<width {
        var sum = 0.0
        for dx in -radius...radius { sum += alpha(min(max(x + dx, 0), width - 1), y) }
        rows[y * width + x] = sum / Double(2 * radius + 1)
    }
}
for y in 0..<height {
    for x in 0..<width {
        var sum = 0.0
        for dy in -radius...radius { sum += rows[min(max(y + dy, 0), height - 1) * width + x] }
        blurred[y * width + x] = sum / Double(2 * radius + 1)
    }
}

let backgroundTop = 48.0, backgroundBottom = 24.0, lines = 236.0, rimBoost = 30.0
for y in 0..<height {
    let progress = min(max(Double(y - top) / Double(max(bottom - top, 1)), 0), 1)
    let background = backgroundTop + (backgroundBottom - backgroundTop) * progress
    for x in 0..<width {
        let i = y * inputRow + x * 4
        let a = Double(input[i + 3])
        let ink = 1 - (0.299 * Double(input[i]) + 0.587 * Double(input[i + 1])
            + 0.114 * Double(input[i + 2])) / 255
        // 1 on the tile, 0 in the shadow around it, which turns black.
        let tile = smoothstep(80, 220, a)
        let rim = smoothstep(0, 1, (250 - blurred[y * width + x]) / 110) * tile
        let base = min(background + rimBoost * rim, 255)
        let gray = UInt8(min(max(((base + (lines - base) * ink) * tile).rounded(), 0), 255))
        let j = (y * width + x) * 4
        output[j] = gray
        output[j + 1] = gray
        output[j + 2] = gray
        output[j + 3] = input[i + 3]
    }
}

// 512px, like src-tauri/icons/icon.png.
let size = 512
let small = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
    samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
    bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: small)
NSGraphicsContext.current!.imageInterpolation = .high
full.draw(
    in: NSRect(x: 0, y: 0, width: size, height: size), from: .zero, operation: .copy,
    fraction: 1, respectFlipped: false, hints: nil)
NSGraphicsContext.restoreGraphicsState()
try! small.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[2]))
