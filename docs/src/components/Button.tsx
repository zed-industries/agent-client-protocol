import clsx from 'clsx'
import Link from 'next/link'

function ArrowIcon(props: React.ComponentPropsWithoutRef<'svg'>) {
  return (
    <svg viewBox="0 0 20 20" fill="none" aria-hidden="true" {...props}>
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m11.5 6.5 3 3.5m0 0-3 3.5m3-3.5h-9"
      />
    </svg>
  )
}

const variantStyles = {
  primary:
    'rounded-full bg-neutral-900 py-1 px-3 text-white hover:bg-neutral-700 dark:bg-teal-400/10 dark:text-teal-400 dark:ring-1 dark:ring-inset dark:ring-teal-400/20 dark:hover:bg-teal-400/10 dark:hover:text-teal-300 dark:hover:ring-teal-300',
  secondary:
    'rounded-full bg-neutral-100 py-1 px-3 text-neutral-900 hover:bg-neutral-200 dark:bg-neutral-800/40 dark:text-neutral-400 dark:ring-1 dark:ring-inset dark:ring-neutral-800 dark:hover:bg-neutral-800 dark:hover:text-neutral-300',
  filled:
    'rounded-full bg-neutral-900 py-1 px-3 text-white hover:bg-neutral-700 dark:bg-teal-500 dark:text-white dark:hover:bg-teal-400',
  outline:
    'rounded-full py-1 px-3 text-neutral-700 ring-1 ring-inset ring-neutral-900/10 hover:bg-neutral-900/2.5 hover:text-neutral-900 dark:text-neutral-400 dark:ring-white/10 dark:hover:bg-white/5 dark:hover:text-white',
  text: 'text-teal-700 hover:text-teal-600 dark:text-teal-400 dark:hover:text-teal-500',
}

type ButtonProps = {
  variant?: keyof typeof variantStyles
  arrow?: 'left' | 'right'
} & (
  | React.ComponentPropsWithoutRef<typeof Link>
  | (React.ComponentPropsWithoutRef<'button'> & { href?: undefined })
)

export function Button({
  variant = 'primary',
  className,
  children,
  arrow,
  ...props
}: ButtonProps) {
  className = clsx(
    'inline-flex gap-0.5 justify-center overflow-hidden text-sm font-medium transition',
    variantStyles[variant],
    className,
  )

  let arrowIcon = (
    <ArrowIcon
      className={clsx(
        'mt-0.5 h-5 w-5',
        variant === 'text' && 'relative top-px',
        arrow === 'left' && '-ml-1 rotate-180',
        arrow === 'right' && '-mr-1',
      )}
    />
  )

  let inner = (
    <>
      {arrow === 'left' && arrowIcon}
      {children}
      {arrow === 'right' && arrowIcon}
    </>
  )

  if (typeof props.href === 'undefined') {
    return (
      <button className={className} {...props}>
        {inner}
      </button>
    )
  }

  return (
    <Link className={className} {...props}>
      {inner}
    </Link>
  )
}
