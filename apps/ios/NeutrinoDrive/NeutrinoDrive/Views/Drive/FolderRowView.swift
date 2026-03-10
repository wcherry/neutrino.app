import SwiftUI

struct FolderRowView: View {
    let folder: FolderItem

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "folder")
                .foregroundColor(.accentColor)
                .frame(width: 28)
            VStack(alignment: .leading, spacing: 4) {
                Text(folder.name)
                    .font(.body)
                    .lineLimit(1)
                if folder.isStarred {
                    Text("Starred")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            Spacer()
            if folder.isStarred {
                Image(systemName: "star.fill")
                    .foregroundColor(.yellow)
            }
        }
    }
}
