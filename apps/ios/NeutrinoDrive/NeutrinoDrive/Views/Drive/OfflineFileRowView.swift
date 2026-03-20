import SwiftUI

struct OfflineFileRowView: View {
    let item: CachedFile

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: iconName)
                .foregroundColor(.accentColor)
                .frame(width: 28)
            VStack(alignment: .leading, spacing: 4) {
                Text(item.name)
                    .font(.body)
                    .lineLimit(1)
                Text("Offline")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            if item.isStarred {
                Image(systemName: "star.fill")
                    .foregroundColor(.yellow)
            }
            Image(systemName: "arrow.down.circle.fill")
                .foregroundColor(.green)
        }
    }

    private var iconName: String {
        if item.mimeType.hasPrefix("image/") { return "photo" }
        if item.mimeType.hasPrefix("video/") { return "film" }
        if item.mimeType.hasPrefix("audio/") { return "music.note" }
        if item.mimeType.contains("pdf") { return "doc.richtext" }
        if item.mimeType.contains("zip") { return "archivebox" }
        return "doc"
    }
}
