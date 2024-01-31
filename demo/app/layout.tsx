import type { Metadata } from 'next';
import { Bricolage_Grotesque, Inter, Roboto_Mono } from 'next/font/google';

import { TailwindIndicator } from '~/components/TailwindIndicator';
import { tw } from '~/utils/tw';

import './global.css';

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-sans',
});

const display = Bricolage_Grotesque({
  subsets: ['latin'],
  variable: '--font-display',
});

export const metadata: Metadata = {
  metadataBase: new URL(`https://${process.env.VERCEL_URL}`),
  title: {
    default: 'New Media',
    template: '%s - Media',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="en"
      className={tw('font-display h-full min-h-dvh antialiased', inter.variable, display.variable)}
    >
      <body>
        {children}
        <TailwindIndicator />
      </body>
    </html>
  );
}
