import SwiftUI

struct FileRowView: View {
    let file: FileItem
    let cached: Bool

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: iconName)
                .foregroundColor(.accentColor)
                .frame(width: 28)
            VStack(alignment: .leading, spacing: 4) {
                Text(file.name)
                    .font(.body)
                    .lineLimit(1)
                Text(fileDetail)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            if file.isStarred {
                Image(systemName: "star.fill")
                    .foregroundColor(.yellow)
            }
            if cached {
                Image(systemName: "arrow.down.circle.fill")
                    .foregroundColor(.green)
            }
        }
    }

    private var iconName: String {
        if file.mimeType.hasPrefix("image/") { return "photo" }
        if file.mimeType.hasPrefix("video/") { return "film" }
        if file.mimeType.hasPrefix("audio/") { return "music.note" }
        if file.mimeType.contains("pdf") { return "doc.richtext" }
        if file.mimeType.contains("zip") { return "archivebox" }
        return "doc" 
    }

    private var fileDetail: String {
        let size = ByteCountFormatter.string(fromByteCount: file.sizeBytes, countStyle: .file)
        return size
    }
}
