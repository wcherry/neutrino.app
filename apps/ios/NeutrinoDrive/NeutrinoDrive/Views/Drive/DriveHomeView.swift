import SwiftUI
import UniformTypeIdentifiers

struct DriveHomeView: View {
    var body: some View {
        NavigationStack {
            DriveFolderView(folderId: nil, title: "My Drive")
        }
    }
}

struct DriveFolderView: View {
    @EnvironmentObject private var drive: DriveService
    @EnvironmentObject private var cache: FileCache

    let folderId: String?
    let title: String

    @State private var filter: DriveFilter = .all
    @State private var folders: [FolderItem] = []
    @State private var files: [FileItem] = []
    @State private var showImporter = false
    @State private var showCreateFolder = false
    @State private var newFolderName = ""
    @State private var errorMessage: String?
    @State private var previewURL: URL?
    @State private var sharingItem: ShareItem?
    @State private var isLoading = false

    var body: some View {
        List {
            if filter != .offline {
                Section("Folders") {
                    if filteredFolders.isEmpty {
                        Text("No folders")
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(filteredFolders) { folder in
                            NavigationLink(value: folder) {
                                FolderRowView(folder: folder)
                            }
                        }
                    }
                }
            }

            Section(filter == .offline ? "Offline Files" : "Files") {
                if filter == .offline {
                    if offlineItems.isEmpty {
                        Text("No offline files")
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(offlineItems, id: \.id) { cached in
                            OfflineFileRowView(item: cached)
                                .contextMenu {
                                    Button("Preview") { previewURL = URL(fileURLWithPath: cached.localPath) }
                                    Button("Remove Offline") { cache.remove(fileId: cached.id) }
                                }
                                .onTapGesture { previewURL = URL(fileURLWithPath: cached.localPath) }
                        }
                    }
                } else {
                    if filteredFiles.isEmpty {
                        Text("No files")
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(filteredFiles) { file in
                            FileRowView(file: file, cached: cache.localURL(for: file.id) != nil)
                                .contextMenu {
                                    Button("Preview") {
                                        openFile(file)
                                    }
                                    Button(file.isStarred ? "Unstar" : "Star") {
                                        Task { await toggleStar(file) }
                                    }
                                    Button(cache.localURL(for: file.id) != nil ? "Remove Offline" : "Download for Offline") {
                                        Task { await toggleOffline(file) }
                                    }
                                    Button("Share") {
                                        shareFile(file)
                                    }
                                }
                                .onTapGesture { openFile(file) }
                        }
                    }
                }
            }
        }
        .navigationTitle(title)
        .navigationDestination(for: FolderItem.self) { folder in
            DriveFolderView(folderId: folder.id, title: folder.name)
        }
        .toolbar {
            ToolbarItem(placement: .navigationBarLeading) {
                Menu {
                    Picker("Filter", selection: $filter) {
                        Text("All").tag(DriveFilter.all)
                        Text("Offline").tag(DriveFilter.offline)
                        Text("Starred").tag(DriveFilter.starred)
                    }
                } label: {
                    Label("Filter", systemImage: "line.3.horizontal.decrease.circle")
                }
            }
            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button("Upload", systemImage: "arrow.up.circle") { showImporter = true }
                    Button("New Folder", systemImage: "folder.badge.plus") { showCreateFolder = true }
                    Button("Refresh", systemImage: "arrow.clockwise") { Task { await reload() } }
                    NavigationLink("Settings", destination: SettingsView())
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
        }
        .fileImporter(
            isPresented: $showImporter,
            allowedContentTypes: [.item],
            allowsMultipleSelection: false
        ) { result in
            handleImport(result)
        }
        .sheet(item: $sharingItem) { item in
            ShareSheet(activityItems: item.items)
        }
        .sheet(item: Binding(get: { previewURL.map { PreviewItem(url: $0) } }, set: { _ in previewURL = nil })) { item in
            QuickLookView(url: item.url)
        }
        .alert("Error", isPresented: Binding(get: { errorMessage != nil }, set: { _ in errorMessage = nil })) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage ?? "")
        }
        .alert("New Folder", isPresented: $showCreateFolder) {
            TextField("Folder name", text: $newFolderName)
            Button("Create") { Task { await createFolder() } }
            Button("Cancel", role: .cancel) { newFolderName = "" }
        }
        .task {
            await reload()
        }
    }

    private var filteredFolders: [FolderItem] {
        switch filter {
        case .all:
            return folders
        case .offline:
            return []
        case .starred:
            return folders.filter { $0.isStarred }
        }
    }

    private var filteredFiles: [FileItem] {
        switch filter {
        case .all:
            return files
        case .offline:
            return []
        case .starred:
            return files.filter { $0.isStarred }
        }
    }

    private var offlineItems: [CachedFile] {
        cache.index.values.sorted { $0.lastAccessed > $1.lastAccessed }
    }

    private func reload() async {
        guard !isLoading else { return }
        isLoading = true
        do {
            let response: FolderContentsResponse
            if let folderId {
                response = try await drive.fetchFolderContents(folderId: folderId)
            } else {
                response = try await drive.fetchRootContents()
            }
            folders = response.folders
            files = response.files
        } catch {
            errorMessage = error.localizedDescription
            NSLog("Error MSG: %@", error.localizedDescription ?? "Unknown error")
        }
        isLoading = false
    }

    private func handleImport(_ result: Result<[URL], Error>) {
        switch result {
        case .success(let urls):
            guard let url = urls.first else { return }
            Task {
                let didStart = url.startAccessingSecurityScopedResource()
                defer { if didStart { url.stopAccessingSecurityScopedResource() } }
                do {
                    _ = try await drive.uploadFile(localURL: url)
                    await reload()
                } catch {
                    errorMessage = error.localizedDescription
                }
            }
        case .failure(let error):
            errorMessage = error.localizedDescription
        }
    }

    private func openFile(_ file: FileItem) {
        Task {
            do {
                let url = try await drive.downloadFile(file)
                previewURL = url
            } catch {
                errorMessage = error.localizedDescription
            }
        }
    }

    private func toggleStar(_ file: FileItem) async {
        do {
            _ = try await drive.toggleStar(file: file)
            await reload()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func toggleOffline(_ file: FileItem) async {
        if cache.localURL(for: file.id) != nil {
            cache.remove(fileId: file.id)
            return
        }
        do {
            _ = try await drive.downloadFile(file)
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func shareFile(_ file: FileItem) {
        if let cached = cache.localURL(for: file.id) {
            sharingItem = ShareItem(items: [cached])
            return
        }
        if let url = drive.getShareURL(for: file) {
            sharingItem = ShareItem(items: [url])
            return
        }
        errorMessage = "Unable to create share link"
    }

    private func createFolder() async {
        let trimmed = newFolderName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            try await drive.createFolder(name: trimmed, parentId: folderId)
            newFolderName = ""
            await reload()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}

enum DriveFilter: Hashable {
    case all
    case offline
    case starred
}

struct PreviewItem: Identifiable {
    let id = UUID()
    let url: URL
}

struct ShareItem: Identifiable {
    let id = UUID()
    let items: [Any]
}
