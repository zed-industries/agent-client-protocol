'use client'

import { Footer } from '@/components/Footer'
import { Header } from '@/components/Header'
import { Logo } from '@/components/Logo'
import { Navigation } from '@/components/Navigation'
import { SectionProvider, type Section } from '@/components/SectionProvider'
import Link from 'next/link'
import { usePathname } from 'next/navigation'

export function Layout({
  children,
  allSections,
}: {
  children: React.ReactNode
  allSections: Record<string, Array<Section>>
}) {
  let pathname = usePathname()

  return (
    <SectionProvider sections={allSections[pathname] ?? []}>
      <div className="size-full lg:ml-72 xl:ml-80">
        <header className="contents lg:pointer-events-none lg:fixed lg:inset-0 lg:z-40 lg:flex">
          <div className="contents lg:pointer-events-auto lg:block lg:w-72 lg:overflow-y-auto lg:border-r lg:border-neutral-900/10 lg:px-6 lg:pt-4 lg:pb-8 xl:w-80 lg:dark:border-neutral-600/12">
            <div className="hidden lg:flex">
              <Link href="/" aria-label="Home">
                <Logo className="h-6" />
              </Link>
            </div>
            <Header />
            <Navigation className="hidden lg:mt-10 lg:block" />
          </div>
        </header>
        <div className="relative flex h-full flex-col px-4 pt-14 sm:px-6 lg:px-8">
          <main className="mx-auto flex size-full max-w-2xl lg:max-w-5xl">
            {children}
          </main>
          <Footer />
        </div>
      </div>
    </SectionProvider>
  )
}
