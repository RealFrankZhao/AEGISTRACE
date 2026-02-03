import AVFoundation
import Foundation

final class Recorder: NSObject, AVCaptureFileOutputRecordingDelegate {
    private let session = AVCaptureSession()
    private let output = AVCaptureMovieFileOutput()
    private let done = DispatchSemaphore(value: 0)

    func record(outputURL: URL, duration: TimeInterval) throws {
        session.sessionPreset = .high

        let screenInput = AVCaptureScreenInput(displayID: CGMainDisplayID())
        screenInput.capturesMouseClicks = true
        screenInput.capturesCursor = true

        if session.canAddInput(screenInput) {
            session.addInput(screenInput)
        } else {
            throw NSError(domain: "aegis", code: 1, userInfo: [NSLocalizedDescriptionKey: "Cannot add screen input"])
        }

        if session.canAddOutput(output) {
            session.addOutput(output)
        } else {
            throw NSError(domain: "aegis", code: 2, userInfo: [NSLocalizedDescriptionKey: "Cannot add movie output"])
        }

        session.startRunning()
        output.startRecording(to: outputURL, recordingDelegate: self)

        DispatchQueue.global().asyncAfter(deadline: .now() + duration) { [weak self] in
            self?.output.stopRecording()
        }

        _ = done.wait(timeout: .now() + duration + 5)
        session.stopRunning()
    }

    func fileOutput(_ output: AVCaptureFileOutput,
                    didFinishRecordingTo outputFileURL: URL,
                    from connections: [AVCaptureConnection],
                    error: Error?) {
        if let error = error {
            fputs("Recording failed: \(error)\n", stderr)
        }
        done.signal()
    }
}

func ensureParentDir(for url: URL) throws {
    let dir = url.deletingLastPathComponent()
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
}

func usage() -> String {
    return "usage: aegis-native-recorder <output_path> <seconds>"
}

let args = CommandLine.arguments.dropFirst()
guard args.count >= 2 else {
    fputs("\(usage())\n", stderr)
    exit(1)
}

let outputPath = String(args[args.startIndex])
let secondsString = String(args[args.startIndex.advanced(by: 1)])

guard let duration = TimeInterval(secondsString), duration > 0 else {
    fputs("invalid duration\n", stderr)
    exit(1)
}

let outputURL = URL(fileURLWithPath: outputPath)

do {
    try ensureParentDir(for: outputURL)
    let recorder = Recorder()
    try recorder.record(outputURL: outputURL, duration: duration)
} catch {
    fputs("FAIL: \(error)\n", stderr)
    exit(1)
}
