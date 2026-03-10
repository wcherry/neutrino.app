import type { Metadata, Viewport } from 'next';
import '@neutrino/ui/styles';
import { ToastProvider } from '@neutrino/ui';
import { QueryProvider } from '@/providers/QueryProvider';

export const metadata: Metadata = {
  title: {
    default: 'Neutrino — Cloud Storage',
    template: '%s | Neutrino',
  },
  description: 'Secure cloud storage for individuals and teams.',
  icons: {
    icon: '/favicon.ico',
  },
};

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" data-app="drive">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>
        <QueryProvider>
          <ToastProvider position="bottom-right">
            {children}
          </ToastProvider>
        </QueryProvider>
      </body>
    </html>
  );
}
