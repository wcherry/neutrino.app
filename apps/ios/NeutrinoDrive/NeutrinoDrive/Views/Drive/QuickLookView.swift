import SwiftUI
import QuickLook
import AVKit
import PDFKit
import UniformTypeIdentifiers
import UIKit
import WebKit

struct QuickLookView: View {
    let url: URL

    @StateObject private var zoom = PreviewZoomModel()

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()

            Group {
                switch previewKind {
                case .image:
                    ZoomableImagePreview(
                        url: url,
                        requestedZoomScale: zoom.zoomScale,
                        minZoom: zoom.minZoom,
                        maxZoom: zoom.maxZoom
                    ) { newZoomScale in
                        zoom.setZoomScale(newZoomScale)
                    }
                case .pdf:
                    ZoomablePDFPreview(url: url, zoom: zoom)
                case .text:
                    ZoomableTextPreview(url: url, zoom: zoom)
                case .document:
                    ZoomableQuickLookDocumentPreview(
                        url: url,
                        requestedZoomScale: zoom.zoomScale,
                        minZoom: zoom.minZoom,
                        maxZoom: zoom.maxZoom
                    ) { newZoomScale in
                        zoom.setZoomScale(newZoomScale)
                    }
                case .video:
                    VideoPreview(url: url)
                case .generic:
                    QuickLookControllerView(url: url)
                }
            }
        }
        .safeAreaInset(edge: .bottom) {
            if previewKind.supportsZoom {
                ZoomControlsBar(zoom: zoom)
                    .padding(.horizontal, 16)
                    .padding(.bottom, 12)
            }
        }
    }

    private var previewKind: PreviewKind {
        PreviewKind(url: url)
    }
}

private enum PreviewKind {
    case image
    case pdf
    case text
    case document
    case video
    case generic

    init(url: URL) {
        let ext = url.pathExtension.lowercased()
        guard let type = UTType(filenameExtension: ext) else {
            if PreviewKind.documentExtensions.contains(ext) {
                self = .document
                return
            }
            self = .generic
            return
        }

        if type.conforms(to: .image) {
            self = .image
        } else if type.conforms(to: .pdf) {
            self = .pdf
        } else if type.conforms(to: .text)
            || type.conforms(to: .plainText)
            || type.conforms(to: .commaSeparatedText)
            || type.conforms(to: .sourceCode)
            || type.conforms(to: .json)
            || type.conforms(to: .xml) {
            self = .text
        } else if PreviewKind.documentExtensions.contains(ext) {
            self = .document
        } else if type.conforms(to: .movie) || type.conforms(to: .video) {
            self = .video
        } else {
            self = .generic
        }
    }

    var supportsZoom: Bool {
        switch self {
        case .image, .pdf, .text, .document:
            return true
        case .video, .generic:
            return false
        }
    }

    private static let documentExtensions: Set<String> = [
        "doc", "docx", "docm",
        "xls", "xlsx", "xlsm",
        "ppt", "pptx", "pptm",
        "rtf", "rtfd",
        "odt", "ods", "odp",
        "pages", "numbers", "key",
        "epub"
    ]
}

@MainActor
private final class PreviewZoomModel: ObservableObject {
    let minZoom: CGFloat = 1
    let maxZoom: CGFloat = 5

    @Published private(set) var zoomScale: CGFloat = 1

    var canZoomIn: Bool {
        zoomScale < maxZoom - 0.01
    }

    var canZoomOut: Bool {
        zoomScale > minZoom + 0.01
    }

    var shouldShowPercentage: Bool {
        abs(zoomScale - 1) > 0.01
    }

    var percentageText: String {
        "\(Int((zoomScale * 100).rounded()))%"
    }

    func zoomIn() {
        setZoomScale(zoomScale * 1.25)
    }

    func zoomOut() {
        setZoomScale(zoomScale / 1.25)
    }

    func setZoomScale(_ newValue: CGFloat) {
        zoomScale = min(max(newValue, minZoom), maxZoom)
    }
}

private struct ZoomControlsBar: View {
    @ObservedObject var zoom: PreviewZoomModel

    var body: some View {
        HStack(spacing: 14) {
            Button {
                zoom.zoomOut()
            } label: {
                Image(systemName: "minus.magnifyingglass")
                    .font(.title3)
            }
            .disabled(!zoom.canZoomOut)

            if zoom.shouldShowPercentage {
                Button {
                    zoom.setZoomScale(1)
                } label: {
                    Text(zoom.percentageText)
                        .font(.subheadline.monospacedDigit())
                        .foregroundColor(.white)
                }
                .buttonStyle(.plain)
                .transition(.opacity)
            }

            Button {
                zoom.zoomIn()
            } label: {
                Image(systemName: "plus.magnifyingglass")
                    .font(.title3)
            }
            .disabled(!zoom.canZoomIn)
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .background(.ultraThinMaterial, in: Capsule())
        .foregroundColor(.white)
        .animation(.easeInOut(duration: 0.15), value: zoom.shouldShowPercentage)
        .animation(.easeInOut(duration: 0.15), value: zoom.zoomScale)
    }
}

private struct ZoomableImagePreview: UIViewRepresentable {
    let url: URL
    let requestedZoomScale: CGFloat
    let minZoom: CGFloat
    let maxZoom: CGFloat
    let onZoomChange: (CGFloat) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(onZoomChange: onZoomChange)
    }

    func makeUIView(context: Context) -> ContainerView {
        let view = ContainerView()
        view.scrollView.delegate = context.coordinator
        context.coordinator.lastAppliedZoomScale = requestedZoomScale
        return view
    }

    func updateUIView(_ view: ContainerView, context: Context) {
        view.scrollView.minimumZoomScale = minZoom
        view.scrollView.maximumZoomScale = maxZoom
        view.updateImageIfNeeded(for: url)
        view.layoutImageIfNeeded()
        context.coordinator.containerView = view
        context.coordinator.applyZoomScaleIfNeeded(to: view.scrollView, zoomScale: requestedZoomScale)
    }

    final class Coordinator: NSObject, UIScrollViewDelegate {
        let onZoomChange: (CGFloat) -> Void
        weak var containerView: ContainerView?
        var lastAppliedZoomScale: CGFloat = 1
        private var isApplyingProgrammaticZoom = false

        init(onZoomChange: @escaping (CGFloat) -> Void) {
            self.onZoomChange = onZoomChange
        }

        func applyZoomScaleIfNeeded(to scrollView: UIScrollView, zoomScale: CGFloat) {
            guard abs(lastAppliedZoomScale - zoomScale) > 0.01 else { return }
            isApplyingProgrammaticZoom = true
            lastAppliedZoomScale = zoomScale
            scrollView.setZoomScale(zoomScale, animated: false)
            DispatchQueue.main.async { [weak self] in
                self?.isApplyingProgrammaticZoom = false
            }
        }

        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            containerView?.imageView
        }

        func scrollViewDidZoom(_ scrollView: UIScrollView) {
            containerView?.updateInsets()
            if !isApplyingProgrammaticZoom {
                lastAppliedZoomScale = scrollView.zoomScale
                DispatchQueue.main.async { [weak self] in
                    self?.onZoomChange(scrollView.zoomScale)
                }
            }
        }
    }

    final class ContainerView: UIView {
        let scrollView = UIScrollView()
        let imageView = UIImageView()

        private var lastLoadedURL: URL?
        private var lastLaidOutURL: URL?
        private var lastBoundsSize: CGSize = .zero

        override init(frame: CGRect) {
            super.init(frame: frame)

            backgroundColor = .black
            scrollView.backgroundColor = .black
            scrollView.showsVerticalScrollIndicator = false
            scrollView.showsHorizontalScrollIndicator = false
            scrollView.bouncesZoom = true

            imageView.contentMode = .scaleAspectFit
            imageView.backgroundColor = .clear

            addSubview(scrollView)
            scrollView.addSubview(imageView)
        }

        required init?(coder: NSCoder) {
            fatalError("init(coder:) has not been implemented")
        }

        override func layoutSubviews() {
            super.layoutSubviews()
            scrollView.frame = bounds
            layoutImageIfNeeded()
        }

        func updateImageIfNeeded(for url: URL) {
            guard lastLoadedURL != url || imageView.image == nil else { return }
            imageView.image = UIImage(contentsOfFile: url.path)
            lastLoadedURL = url
            lastLaidOutURL = nil
            setNeedsLayout()
        }

        func layoutImageIfNeeded() {
            guard lastBoundsSize != scrollView.bounds.size || lastLaidOutURL != lastLoadedURL else {
                updateInsets()
                return
            }
            layoutImage()
        }

        func layoutImage() {
            guard let image = imageView.image else { return }
            let boundsSize = scrollView.bounds.size
            guard boundsSize.width > 0, boundsSize.height > 0 else { return }
            guard image.size.width > 0, image.size.height > 0 else { return }

            let widthScale = boundsSize.width / image.size.width
            let heightScale = boundsSize.height / image.size.height
            let fitScale = min(widthScale, heightScale)
            let fittedSize = CGSize(
                width: image.size.width * fitScale,
                height: image.size.height * fitScale
            )

            imageView.frame = CGRect(origin: .zero, size: fittedSize)
            scrollView.contentSize = fittedSize
            if scrollView.zoomScale <= 1.01 {
                scrollView.contentOffset = .zero
            }
            lastBoundsSize = boundsSize
            lastLaidOutURL = lastLoadedURL
            updateInsets()
        }

        func updateInsets() {
            let horizontalInset = max((scrollView.bounds.width - scrollView.contentSize.width) * 0.5, 0)
            let verticalInset = max((scrollView.bounds.height - scrollView.contentSize.height) * 0.5, 0)
            scrollView.contentInset = UIEdgeInsets(
                top: verticalInset,
                left: horizontalInset,
                bottom: verticalInset,
                right: horizontalInset
            )
        }
    }
}

private struct ZoomablePDFPreview: UIViewRepresentable {
    let url: URL
    @ObservedObject var zoom: PreviewZoomModel

    func makeCoordinator() -> Coordinator {
        Coordinator(zoom: zoom)
    }

    func makeUIView(context: Context) -> PDFView {
        let pdfView = PDFView()
        pdfView.backgroundColor = .black
        pdfView.displayMode = .singlePageContinuous
        pdfView.displayDirection = .vertical
        pdfView.autoScales = true
        pdfView.document = PDFDocument(url: url)

        context.coordinator.attach(to: pdfView)
        context.coordinator.configureScale(for: pdfView)
        return pdfView
    }

    func updateUIView(_ pdfView: PDFView, context: Context) {
        if pdfView.document?.documentURL != url {
            pdfView.document = PDFDocument(url: url)
            context.coordinator.configureScale(for: pdfView)
        }

        context.coordinator.applyZoomScaleIfNeeded(to: pdfView, zoomScale: zoom.zoomScale)
    }

    final class Coordinator {
        let zoom: PreviewZoomModel
        private var observation: NSObjectProtocol?
        private var isApplyingProgrammaticZoom = false

        init(zoom: PreviewZoomModel) {
            self.zoom = zoom
        }

        deinit {
            if let observation {
                NotificationCenter.default.removeObserver(observation)
            }
        }

        func attach(to pdfView: PDFView) {
            if let observation {
                NotificationCenter.default.removeObserver(observation)
            }

            observation = NotificationCenter.default.addObserver(
                forName: Notification.Name.PDFViewScaleChanged,
                object: pdfView,
                queue: .main
            ) { [weak self, weak pdfView] _ in
                guard let self, let pdfView else { return }
                guard !self.isApplyingProgrammaticZoom else { return }
                let baseScale = pdfView.scaleFactorForSizeToFit
                guard baseScale > 0 else { return }
                DispatchQueue.main.async { [weak self] in
                    self?.zoom.setZoomScale(pdfView.scaleFactor / baseScale)
                }
            }
        }

        func configureScale(for pdfView: PDFView) {
            DispatchQueue.main.async {
                let fitScale = pdfView.scaleFactorForSizeToFit
                guard fitScale > 0 else { return }
                pdfView.minScaleFactor = fitScale
                pdfView.maxScaleFactor = fitScale * self.zoom.maxZoom
                pdfView.scaleFactor = fitScale * self.zoom.zoomScale
            }
        }

        func applyZoomScaleIfNeeded(to pdfView: PDFView, zoomScale: CGFloat) {
            let baseScale = pdfView.scaleFactorForSizeToFit
            guard baseScale > 0 else { return }

            let desiredScale = baseScale * zoomScale
            guard abs(pdfView.scaleFactor - desiredScale) > 0.01 else { return }

            isApplyingProgrammaticZoom = true
            pdfView.scaleFactor = desiredScale
            DispatchQueue.main.async { [weak self] in
                self?.isApplyingProgrammaticZoom = false
            }
        }
    }
}

private struct ZoomableTextPreview: UIViewRepresentable {
    let url: URL
    @ObservedObject var zoom: PreviewZoomModel

    func makeUIView(context: Context) -> UITextView {
        let textView = UITextView()
        textView.backgroundColor = .black
        textView.textColor = .white
        textView.isEditable = false
        textView.alwaysBounceVertical = true
        textView.textContainerInset = UIEdgeInsets(top: 20, left: 16, bottom: 20, right: 16)
        textView.adjustsFontForContentSizeCategory = true
        update(textView)
        return textView
    }

    func updateUIView(_ textView: UITextView, context: Context) {
        update(textView)
    }

    private func update(_ textView: UITextView) {
        textView.font = UIFont.monospacedSystemFont(ofSize: 16 * zoom.zoomScale, weight: .regular)
        textView.text = loadText()
    }

    private func loadText() -> String {
        if let text = try? String(contentsOf: url, encoding: .utf8) {
            return text
        }
        if let text = try? String(contentsOf: url, encoding: .ascii) {
            return text
        }
        if let data = try? Data(contentsOf: url),
           let text = String(data: data, encoding: .utf16) {
            return text
        }
        return "Preview unavailable."
    }
}

private struct ZoomableQuickLookDocumentPreview: UIViewControllerRepresentable {
    let url: URL
    let requestedZoomScale: CGFloat
    let minZoom: CGFloat
    let maxZoom: CGFloat
    let onZoomChange: (CGFloat) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(url: url, onZoomChange: onZoomChange)
    }

    func makeUIViewController(context: Context) -> ContainerViewController {
        let controller = ContainerViewController()
        context.coordinator.installPreview(in: controller)
        context.coordinator.configureZoom(in: controller, minZoom: minZoom, maxZoom: maxZoom)
        context.coordinator.applyZoomIfNeeded(in: controller, zoomScale: requestedZoomScale)
        return controller
    }

    func updateUIViewController(_ uiViewController: ContainerViewController, context: Context) {
        context.coordinator.updateURLIfNeeded(url, in: uiViewController)
        context.coordinator.configureZoom(in: uiViewController, minZoom: minZoom, maxZoom: maxZoom)
        context.coordinator.applyZoomIfNeeded(in: uiViewController, zoomScale: requestedZoomScale)
    }

    final class Coordinator: NSObject, QLPreviewControllerDataSource, UIScrollViewDelegate {
        var url: URL
        let onZoomChange: (CGFloat) -> Void
        private let previewController = QLPreviewController()
        weak var container: ContainerViewController?
        private var lastAppliedZoom: CGFloat = 1
        private var isApplyingProgrammaticZoom = false

        init(url: URL, onZoomChange: @escaping (CGFloat) -> Void) {
            self.url = url
            self.onZoomChange = onZoomChange
            super.init()
            previewController.dataSource = self
        }

        func numberOfPreviewItems(in controller: QLPreviewController) -> Int { 1 }

        func previewController(_ controller: QLPreviewController, previewItemAt index: Int) -> QLPreviewItem {
            url as QLPreviewItem
        }

        func installPreview(in container: ContainerViewController) {
            self.container = container
            guard previewController.parent == nil else {
                container.attachPreviewViewIfNeeded(previewController.view)
                container.scrollView.delegate = self
                return
            }

            container.scrollView.delegate = self
            container.addChild(previewController)
            container.attachPreviewViewIfNeeded(previewController.view)
            previewController.didMove(toParent: container)
        }

        func updateURLIfNeeded(_ newURL: URL, in container: ContainerViewController) {
            guard url != newURL else { return }
            url = newURL
            previewController.reloadData()
            container.resetZoom()
            lastAppliedZoom = 1
        }

        func configureZoom(in container: ContainerViewController, minZoom: CGFloat, maxZoom: CGFloat) {
            container.scrollView.minimumZoomScale = minZoom
            container.scrollView.maximumZoomScale = maxZoom
            container.updateInsets()
        }

        func applyZoomIfNeeded(in container: ContainerViewController, zoomScale: CGFloat) {
            guard abs(lastAppliedZoom - zoomScale) > 0.01 else { return }
            isApplyingProgrammaticZoom = true
            lastAppliedZoom = zoomScale
            container.scrollView.setZoomScale(zoomScale, animated: false)
            container.updateInsets()
            DispatchQueue.main.async { [weak self] in
                self?.isApplyingProgrammaticZoom = false
            }
        }

        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            container?.zoomContentView
        }

        func scrollViewDidZoom(_ scrollView: UIScrollView) {
            container?.updateInsets()
            if !isApplyingProgrammaticZoom {
                lastAppliedZoom = scrollView.zoomScale
                DispatchQueue.main.async { [weak self] in
                    self?.onZoomChange(scrollView.zoomScale)
                }
            }
        }
    }

    final class ContainerViewController: UIViewController {
        let scrollView = UIScrollView()
        let zoomContentView = UIView()
        private weak var previewView: UIView?

        override func viewDidLoad() {
            super.viewDidLoad()

            view.backgroundColor = .black
            scrollView.backgroundColor = .black
            scrollView.showsVerticalScrollIndicator = false
            scrollView.showsHorizontalScrollIndicator = false
            scrollView.bouncesZoom = true

            view.addSubview(scrollView)
            scrollView.addSubview(zoomContentView)
        }

        override func viewDidLayoutSubviews() {
            super.viewDidLayoutSubviews()

            scrollView.frame = view.bounds
            if zoomContentView.frame.size != scrollView.bounds.size {
                zoomContentView.frame = CGRect(origin: .zero, size: scrollView.bounds.size)
                previewView?.frame = zoomContentView.bounds
                scrollView.contentSize = zoomContentView.bounds.size
                if scrollView.zoomScale <= 1.01 {
                    scrollView.contentOffset = .zero
                }
            } else {
                previewView?.frame = zoomContentView.bounds
            }

            updateInsets()
        }

        func attachPreviewViewIfNeeded(_ view: UIView) {
            guard previewView !== view else { return }
            previewView?.removeFromSuperview()
            previewView = view
            zoomContentView.addSubview(view)
            view.frame = zoomContentView.bounds
            scrollView.contentSize = zoomContentView.bounds.size
        }

        func updateInsets() {
            let horizontalInset = max((scrollView.bounds.width - scrollView.contentSize.width) * 0.5, 0)
            let verticalInset = max((scrollView.bounds.height - scrollView.contentSize.height) * 0.5, 0)
            scrollView.contentInset = UIEdgeInsets(
                top: verticalInset,
                left: horizontalInset,
                bottom: verticalInset,
                right: horizontalInset
            )
        }

        func resetZoom() {
            scrollView.setZoomScale(1, animated: false)
            scrollView.contentOffset = .zero
            updateInsets()
        }
    }
}

struct NeutrinoEditorPreviewView: View {
    let payload: NeutrinoPreviewPayload

    var body: some View {
        NavigationStack {
            NeutrinoPreviewWebView(html: html)
                .background(Color(.systemBackground))
                .navigationTitle(payload.title)
                .navigationBarTitleDisplayMode(.inline)
        }
    }

    private var html: String {
        let title = payload.title.htmlEscaped
        let kind = payload.kind.rawValue.htmlEscaped
        let contentBase64 = Data(payload.content.utf8).base64EncodedString()

        return """
        <!DOCTYPE html>
        <html>
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=5.0" />
          <title>\(title)</title>
          <style>
            :root {
              color-scheme: light;
              --bg: #f3f5f9;
              --panel: #ffffff;
              --muted: #6b7280;
              --text: #111827;
              --border: #dbe3ee;
              --accent: #1d4ed8;
            }
            * { box-sizing: border-box; }
            body {
              margin: 0;
              padding: 20px 16px 48px;
              background: linear-gradient(180deg, #eef3fb 0%, var(--bg) 100%);
              color: var(--text);
              font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif;
            }
            .shell {
              max-width: 980px;
              margin: 0 auto;
            }
            .header {
              margin-bottom: 16px;
            }
            .badge {
              display: inline-flex;
              align-items: center;
              gap: 8px;
              padding: 6px 10px;
              border-radius: 999px;
              background: rgba(29, 78, 216, 0.08);
              color: var(--accent);
              font-size: 12px;
              font-weight: 700;
              letter-spacing: 0.04em;
              text-transform: uppercase;
            }
            h1 {
              margin: 12px 0 6px;
              font-size: 28px;
              line-height: 1.15;
            }
            .subtitle {
              margin: 0;
              color: var(--muted);
              font-size: 14px;
            }
            .doc {
              background: var(--panel);
              border: 1px solid var(--border);
              border-radius: 24px;
              box-shadow: 0 20px 50px rgba(15, 23, 42, 0.08);
              overflow: hidden;
            }
            .doc-body {
              padding: 28px 22px;
            }
            .doc-body h1, .doc-body h2, .doc-body h3, .doc-body h4, .doc-body h5, .doc-body h6 {
              margin: 1.2em 0 0.4em;
            }
            .doc-body p, .doc-body ul, .doc-body ol, .doc-body blockquote, .doc-body pre {
              margin: 0.75em 0;
              line-height: 1.6;
            }
            .doc-body table {
              width: 100%;
              border-collapse: collapse;
              margin: 16px 0;
            }
            .doc-body th, .doc-body td {
              border: 1px solid var(--border);
              padding: 10px 12px;
              vertical-align: top;
            }
            .doc-body th {
              background: #f8fafc;
              text-align: left;
            }
            .doc-body blockquote {
              padding-left: 14px;
              border-left: 4px solid #bfdbfe;
              color: #374151;
            }
            .doc-body pre {
              padding: 14px;
              border-radius: 14px;
              background: #0f172a;
              color: #e2e8f0;
              overflow: auto;
            }
            .doc-body img {
              max-width: 100%;
              border-radius: 14px;
            }
            .sheet-frame {
              background: var(--panel);
              border: 1px solid var(--border);
              border-radius: 24px;
              box-shadow: 0 20px 50px rgba(15, 23, 42, 0.08);
              overflow: auto;
            }
            .sheet-meta {
              display: flex;
              justify-content: space-between;
              align-items: center;
              padding: 16px 20px;
              border-bottom: 1px solid var(--border);
              background: linear-gradient(180deg, #f8fbff 0%, #f1f5f9 100%);
              color: var(--muted);
              font-size: 13px;
            }
            .sheet-table {
              width: max-content;
              min-width: 100%;
              border-collapse: collapse;
            }
            .sheet-table th, .sheet-table td {
              min-width: 120px;
              max-width: 240px;
              padding: 10px 12px;
              border: 1px solid var(--border);
              white-space: pre-wrap;
              word-break: break-word;
            }
            .sheet-table th {
              position: sticky;
              top: 0;
              background: #f8fafc;
              z-index: 1;
              text-align: left;
            }
            .slides {
              display: grid;
              gap: 18px;
            }
            .slide {
              position: relative;
              width: 100%;
              aspect-ratio: 16 / 9;
              border-radius: 24px;
              overflow: hidden;
              background: #fff;
              border: 1px solid var(--border);
              box-shadow: 0 18px 44px rgba(15, 23, 42, 0.10);
            }
            .slide-number {
              position: absolute;
              top: 14px;
              right: 16px;
              padding: 6px 10px;
              border-radius: 999px;
              background: rgba(255, 255, 255, 0.88);
              border: 1px solid rgba(148, 163, 184, 0.35);
              color: #334155;
              font-size: 12px;
              font-weight: 700;
            }
            .slide-el {
              position: absolute;
              white-space: pre-wrap;
              overflow: hidden;
            }
            .empty {
              padding: 48px 24px;
              text-align: center;
              color: var(--muted);
            }
          </style>
        </head>
        <body>
          <div class="shell">
            <div class="header">
              <div class="badge">Neutrino \(kind)</div>
              <h1>\(title)</h1>
              <p class="subtitle">Mobile preview</p>
            </div>
            <div id="root"></div>
          </div>
          <script>
            const kind = "\(kind)";
            const raw = decodeURIComponent(escape(atob("\(contentBase64)")));

            function safeJsonParse(value) {
              try { return JSON.parse(value); } catch { return null; }
            }

            function escapeHtml(value) {
              return String(value ?? '')
                .replaceAll('&', '&amp;')
                .replaceAll('<', '&lt;')
                .replaceAll('>', '&gt;')
                .replaceAll('"', '&quot;')
                .replaceAll("'", '&#39;');
            }

            function renderMarks(text, marks) {
              let html = escapeHtml(text);
              for (const mark of marks ?? []) {
                const type = mark.type;
                if (type === 'bold') html = `<strong>${html}</strong>`;
                else if (type === 'italic') html = `<em>${html}</em>`;
                else if (type === 'underline') html = `<u>${html}</u>`;
                else if (type === 'strike') html = `<s>${html}</s>`;
                else if (type === 'code') html = `<code>${html}</code>`;
                else if (type === 'link') {
                  const href = escapeHtml(mark.attrs?.href ?? '#');
                  html = `<a href="${href}">${html}</a>`;
                }
              }
              return html;
            }

            function renderDocNode(node) {
              if (!node) return '';
              const children = (node.content ?? []).map(renderDocNode).join('');
              switch (node.type) {
                case 'doc':
                  return children;
                case 'paragraph':
                  return `<p>${children || '&nbsp;'}</p>`;
                case 'text':
                  return renderMarks(node.text ?? '', node.marks);
                case 'heading': {
                  const level = Math.min(Math.max(Number(node.attrs?.level ?? 1), 1), 6);
                  return `<h${level}>${children}</h${level}>`;
                }
                case 'bulletList':
                  return `<ul>${children}</ul>`;
                case 'orderedList':
                  return `<ol>${children}</ol>`;
                case 'listItem':
                  return `<li>${children}</li>`;
                case 'blockquote':
                  return `<blockquote>${children}</blockquote>`;
                case 'codeBlock':
                  return `<pre><code>${children}</code></pre>`;
                case 'horizontalRule':
                  return '<hr />';
                case 'hardBreak':
                  return '<br />';
                case 'image': {
                  const src = escapeHtml(node.attrs?.src ?? '');
                  const alt = escapeHtml(node.attrs?.alt ?? '');
                  return src ? `<img src="${src}" alt="${alt}" />` : '';
                }
                case 'table':
                  return `<table><tbody>${children}</tbody></table>`;
                case 'tableRow':
                  return `<tr>${children}</tr>`;
                case 'tableHeader':
                  return `<th>${children}</th>`;
                case 'tableCell':
                  return `<td>${children}</td>`;
                default:
                  return children;
              }
            }

            function renderDoc(content) {
              const parsed = safeJsonParse(content);
              const body = parsed ? renderDocNode(parsed) : `<pre>${escapeHtml(content)}</pre>`;
              return `<article class="doc"><div class="doc-body">${body || '<div class="empty">This document is empty.</div>'}</div></article>`;
            }

            function cellDisplayValue(cell) {
              if (!cell) return '';
              if (typeof cell.m === 'string' && cell.m.length > 0) return cell.m;
              if (typeof cell.w === 'string' && cell.w.length > 0) return cell.w;
              if (typeof cell.f === 'string' && cell.f.length > 0) return cell.f;
              if (cell.v === null || cell.v === undefined) return '';
              return String(cell.v);
            }

            function columnLabel(index) {
              let label = '';
              let value = Number(index);
              while (value >= 0) {
                label = String.fromCharCode(65 + (value % 26)) + label;
                value = Math.floor(value / 26) - 1;
              }
              return label;
            }

            function collectSheetCells(sheet) {
              const cells = new Map();
              let maxRow = 0;
              let maxCol = 0;

              if (Array.isArray(sheet?.celldata) && sheet.celldata.length > 0) {
                for (const item of sheet.celldata) {
                  const row = Number(item?.r ?? 0);
                  const col = Number(item?.c ?? 0);
                  cells.set(`${row}:${col}`, cellDisplayValue(item?.v));
                  maxRow = Math.max(maxRow, row);
                  maxCol = Math.max(maxCol, col);
                }
                return { cells, maxRow, maxCol };
              }

              if (Array.isArray(sheet?.data) && sheet.data.length > 0) {
                for (let rowIndex = 0; rowIndex < sheet.data.length; rowIndex += 1) {
                  const row = Array.isArray(sheet.data[rowIndex]) ? sheet.data[rowIndex] : [];
                  for (let colIndex = 0; colIndex < row.length; colIndex += 1) {
                    const value = cellDisplayValue(row[colIndex]);
                    if (value === '') continue;
                    cells.set(`${rowIndex}:${colIndex}`, value);
                    maxRow = Math.max(maxRow, rowIndex);
                    maxCol = Math.max(maxCol, colIndex);
                  }
                }
                return { cells, maxRow, maxCol };
              }

              return { cells, maxRow, maxCol };
            }

            function renderSheet(content) {
              const parsed = safeJsonParse(content);
              const sheets = Array.isArray(parsed) ? parsed : [];
              const active = sheets.find(sheet => sheet.status === 1) ?? sheets[0];
              if (!active) {
                return '<div class="empty">This spreadsheet is empty.</div>';
              }

              const extracted = collectSheetCells(active);
              const cells = extracted.cells;
              let maxRow = extracted.maxRow;
              let maxCol = extracted.maxCol;

              maxRow = Math.min(Math.max(maxRow + 1, Number(active.row ?? 12), 12), 120);
              maxCol = Math.min(Math.max(maxCol + 1, Number(active.column ?? 8), 8), 52);

              const header = Array.from({ length: maxCol }, (_, index) => {
                const label = columnLabel(index);
                return `<th>${label}</th>`;
              }).join('');

              const rows = Array.from({ length: maxRow }, (_, rowIndex) => {
                const cellsHtml = Array.from({ length: maxCol }, (_, colIndex) => {
                  const key = `${rowIndex}:${colIndex}`;
                  return `<td>${escapeHtml(cells.get(key) ?? '')}</td>`;
                }).join('');
                return `<tr><th>${rowIndex + 1}</th>${cellsHtml}</tr>`;
              }).join('');

              return `
                <section class="sheet-frame">
                  <div class="sheet-meta">
                    <span>${escapeHtml(active.name ?? 'Sheet')}</span>
                    <span>${maxRow} rows • ${maxCol} columns</span>
                  </div>
                  <table class="sheet-table">
                    <thead><tr><th></th>${header}</tr></thead>
                    <tbody>${rows}</tbody>
                  </table>
                </section>
              `;
            }

            function slideElementHtml(element) {
              const x = Number(element.x ?? 0);
              const y = Number(element.y ?? 0);
              const w = Number(element.w ?? 10);
              const h = Number(element.h ?? 10);
              const style = element.style ?? {};
              const common = `
                left:${x}%;
                top:${y}%;
                width:${w}%;
                height:${h}%;
                color:${style.color ?? '#111827'};
                font-size:${Number(style.fontSize ?? 18)}px;
                font-family:${style.fontFamily ?? '-apple-system'};
                font-weight:${style.bold ? 700 : 400};
                font-style:${style.italic ? 'italic' : 'normal'};
                text-decoration:${style.underline ? 'underline' : 'none'};
                text-align:${style.align ?? 'left'};
              `;

              if (element.type === 'text') {
                return `<div class="slide-el" style="${common}">${escapeHtml(element.content ?? '')}</div>`;
              }
              return '';
            }

            function renderSlides(content) {
              const parsed = safeJsonParse(content);
              const slides = Array.isArray(parsed?.slides) ? parsed.slides : [];
              if (!slides.length) {
                return '<div class="empty">This presentation has no slides yet.</div>';
              }

              return `<section class="slides">${slides.map((slide, index) => {
                const background = slide.background?.type === 'color'
                  ? (slide.background?.value ?? '#ffffff')
                  : '#ffffff';
                const elements = (slide.elements ?? []).map(slideElementHtml).join('');
                return `
                  <article class="slide" style="background:${escapeHtml(background)}">
                    <div class="slide-number">Slide ${index + 1}</div>
                    ${elements}
                  </article>
                `;
              }).join('')}</section>`;
            }

            const root = document.getElementById('root');
            if (kind === 'doc') root.innerHTML = renderDoc(raw);
            else if (kind === 'sheet') root.innerHTML = renderSheet(raw);
            else if (kind === 'slide') root.innerHTML = renderSlides(raw);
            else root.innerHTML = `<pre>${escapeHtml(raw)}</pre>`;
          </script>
        </body>
        </html>
        """
    }
}

private struct NeutrinoPreviewWebView: UIViewRepresentable {
    let html: String

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = true

        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.isOpaque = false
        webView.backgroundColor = .clear
        webView.scrollView.backgroundColor = .clear
        webView.scrollView.contentInsetAdjustmentBehavior = .never
        context.coordinator.loadIfNeeded(html, into: webView)
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        context.coordinator.loadIfNeeded(html, into: webView)
    }

    final class Coordinator {
        private var lastHTML: String?

        func loadIfNeeded(_ html: String, into webView: WKWebView) {
            guard lastHTML != html else { return }
            lastHTML = html
            webView.loadHTMLString(html, baseURL: nil)
        }
    }
}

private struct VideoPreview: View {
    let url: URL

    @State private var player: AVPlayer

    init(url: URL) {
        self.url = url
        _player = State(initialValue: AVPlayer(url: url))
    }

    var body: some View {
        VideoPlayer(player: player)
            .background(Color.black)
            .onAppear { player.play() }
            .onDisappear { player.pause() }
    }
}

private extension String {
    var htmlEscaped: String {
        self
            .replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
            .replacingOccurrences(of: "\"", with: "&quot;")
            .replacingOccurrences(of: "'", with: "&#39;")
    }
}

private struct QuickLookControllerView: UIViewControllerRepresentable {
    let url: URL

    func makeUIViewController(context: Context) -> QLPreviewController {
        let controller = QLPreviewController()
        controller.dataSource = context.coordinator
        return controller
    }

    func updateUIViewController(_ uiViewController: QLPreviewController, context: Context) {
        context.coordinator.url = url
        uiViewController.reloadData()
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(url: url)
    }

    final class Coordinator: NSObject, QLPreviewControllerDataSource {
        var url: URL

        init(url: URL) {
            self.url = url
        }

        func numberOfPreviewItems(in controller: QLPreviewController) -> Int { 1 }

        func previewController(_ controller: QLPreviewController, previewItemAt index: Int) -> QLPreviewItem {
            url as QLPreviewItem
        }
    }
}
