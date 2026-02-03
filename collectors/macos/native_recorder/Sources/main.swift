import AVFoundation
import Foundation

final class Recorder: NSObject, AVCaptureFileOutputRecordingDelegate {
    private let session = AVCaptureSession()
    private let output = AVCaptureMovieFileOutput()
    private let done = DispatchSemaphore(value: 0)
    private var signalSources: [DispatchSourceSignal] = []

    func record(outputURL: URL, duration: TimeInterval) throws {
        session.sessionPreset = .high

        guard let screenInput = AVCaptureScreenInput(displayID: CGMainDisplayID()) else {
            throw NSError(domain: "aegis", code: 0, userInfo: [NSLocalizedDescriptionKey: "Cannot create screen input"])
        }
        screenInput.capturesMouseClicks = true
        screenInput.capturesCursor = true
        let displayBounds = CGDisplayBounds(CGMainDisplayID())
        let targetSize = CGSize(width: 1280, height: 720)
        let scale = min(targetSize.width / displayBounds.width, targetSize.height / displayBounds.height)
        if scale < 1 {
            screenInput.scaleFactor = scale
        }
        let fps = 30
        screenInput.minFrameDuration = CMTimeMake(value: 1, timescale: Int32(fps))

        if session.canAddInput(screenInput) {
            session.addInput(screenInput)
        } else {
            throw NSError(domain: "aegis", code: 1, userInfo: [NSLocalizedDescriptionKey: "Cannot add screen input"])
        }

        if session.canAddOutput(output) {
            session.addOutput(output)
            if let connection = output.connection(with: .video) {
                output.setOutputSettings(
                    [
                        AVVideoCodecKey: AVVideoCodecType.hevc,
                        AVVideoCompressionPropertiesKey: [
                            AVVideoAverageBitRateKey: 2_000_000,
                            AVVideoMaxKeyFrameIntervalKey: fps * 2
                        ]
                    ],
                    for: connection
                )
            }
        } else {
            throw NSError(domain: "aegis", code: 2, userInfo: [NSLocalizedDescriptionKey: "Cannot add movie output"])
        }

        setupSignalHandlers()
        session.startRunning()
        output.startRecording(to: outputURL, recordingDelegate: self)

        DispatchQueue.global().async { [weak self] in
            _ = FileHandle.standardInput.readDataToEndOfFile()
            self?.output.stopRecording()
        }

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

    private func setupSignalHandlers() {
        let signals: [Int32] = [SIGINT, SIGTERM]
        for sig in signals {
            signal(sig, SIG_IGN)
            let source = DispatchSource.makeSignalSource(signal: sig, queue: DispatchQueue.global())
            source.setEventHandler { [weak self] in
                self?.output.stopRecording()
            }
            source.resume()
            signalSources.append(source)
        }
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
