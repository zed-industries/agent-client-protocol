'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

import { Button } from '@/components/Button'
import { navigation } from '@/components/Navigation'

function PageLink({
  label,
  page,
  previous = false,
}: {
  label: string
  page: { href: string; title: string }
  previous?: boolean
}) {
  return (
    <>
      <Button
        href={page.href}
        aria-label={`${label}: ${page.title}`}
        variant="text"
        arrow={previous ? 'left' : 'right'}
      >
        {label}
      </Button>
      <Link
        href={page.href}
        tabIndex={-1}
        aria-hidden="true"
        className="font-semibold text-neutral-900 transition hover:text-neutral-600 dark:text-white dark:hover:text-neutral-300"
      >
        {page.title}
      </Link>
    </>
  )
}

export function Footer() {
  let pathname = usePathname()
  let allPages = navigation.flatMap((group) => group.links)
  let currentPageIndex = allPages.findIndex((page) => page.href === pathname)

  if (currentPageIndex === -1) {
    return null
  }

  let previousPage = allPages[currentPageIndex - 1]
  let nextPage = allPages[currentPageIndex + 1]

  if (!previousPage && !nextPage) {
    return null
  }

  return (
    <footer className="mx-auto flex w-full max-w-2xl flex-col space-y-10 pb-8 lg:max-w-5xl">
      <div className="mx-auto flex w-full max-w-[800px] border-t border-neutral-200 pt-8 dark:border-neutral-600/20">
        {previousPage && (
          <div className="flex flex-col items-start gap-3">
            <PageLink label="Previous" page={previousPage} previous />
          </div>
        )}
        {nextPage && (
          <div className="ml-auto flex flex-col items-end gap-3">
            <PageLink label="Next" page={nextPage} />
          </div>
        )}
      </div>
    </footer>
  )
}
