import * as DropdownMenu from '@radix-ui/react-dropdown-menu'
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useState,
} from 'react'
import { Drawer } from 'vaul'
import { MOBILE_BREAKPOINT_QUERY, useMediaQuery } from '../hooks/useMediaQuery'
import './ResponsiveMenu.css'

/**
 * A dropdown menu that renders as a Radix `DropdownMenu` on desktop and, on
 * narrow viewports, as a full-screen `vaul` drawer instead — the small
 * floating panel a `DropdownMenu` renders doesn't work as a touch target on
 * mobile. Mirrors just the slice of the `DropdownMenu` API this codebase
 * actually uses (`Root`/`Trigger`/`Content`/`Item`/`Label`), so call sites
 * that already use `DropdownMenu` can switch over with a find/replace.
 */

interface ResponsiveMenuContextValue {
  isMobile: boolean
  setOpen: (open: boolean) => void
}

const ResponsiveMenuContext = createContext<ResponsiveMenuContextValue | null>(
  null,
)

function useResponsiveMenuContext(component: string) {
  const context = useContext(ResponsiveMenuContext)
  if (!context) {
    throw new Error(
      `<ResponsiveMenu.${component}> must be used inside <ResponsiveMenu.Root>`,
    )
  }
  return context
}

interface RootProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  /** Forwarded to the desktop `DropdownMenu.Root` only — vaul's drawer is
   * always modal. */
  modal?: boolean
  children: ReactNode
}

function Root({
  open: openProp,
  onOpenChange,
  modal = false,
  children,
}: RootProps) {
  const isMobile = useMediaQuery(MOBILE_BREAKPOINT_QUERY)
  const [uncontrolledOpen, setUncontrolledOpen] = useState(false)
  const open = openProp ?? uncontrolledOpen

  const setOpen = useCallback(
    (next: boolean) => {
      if (openProp === undefined) setUncontrolledOpen(next)
      onOpenChange?.(next)
    },
    [openProp, onOpenChange],
  )

  const contextValue = useMemo<ResponsiveMenuContextValue>(
    () => ({ isMobile, setOpen }),
    [isMobile, setOpen],
  )

  return (
    <ResponsiveMenuContext.Provider value={contextValue}>
      {isMobile ? (
        <Drawer.Root open={open} onOpenChange={setOpen}>
          {children}
        </Drawer.Root>
      ) : (
        <DropdownMenu.Root modal={modal} open={open} onOpenChange={setOpen}>
          {children}
        </DropdownMenu.Root>
      )}
    </ResponsiveMenuContext.Provider>
  )
}

interface TriggerProps {
  asChild?: boolean
  children: ReactNode
}

function Trigger({ asChild, children }: TriggerProps) {
  const { isMobile } = useResponsiveMenuContext('Trigger')
  return isMobile ? (
    <Drawer.Trigger asChild={asChild}>{children}</Drawer.Trigger>
  ) : (
    <DropdownMenu.Trigger asChild={asChild}>{children}</DropdownMenu.Trigger>
  )
}

interface ContentProps {
  className?: string
  align?: 'start' | 'center' | 'end'
  sideOffset?: number
  /** Heading for the drawer this renders as on mobile, so the viewer knows
   * which menu they're in without the trigger button visible behind it;
   * unused on desktop, where the trigger stays right next to the menu. */
  title: string
  children: ReactNode
}

function Content({
  className,
  align,
  sideOffset,
  title,
  children,
}: ContentProps) {
  const { isMobile } = useResponsiveMenuContext('Content')

  if (isMobile) {
    return (
      <Drawer.Portal>
        <Drawer.Overlay className="responsive-menu-overlay" />
        <Drawer.Content className="responsive-menu-drawer">
          <div className="responsive-menu-drawer-header">
            <div className="responsive-menu-drawer-handle" aria-hidden="true" />
            <Drawer.Title className="responsive-menu-drawer-title">
              {title}
            </Drawer.Title>
          </div>
          <div className={`responsive-menu-drawer-body ${className ?? ''}`}>
            {children}
          </div>
        </Drawer.Content>
      </Drawer.Portal>
    )
  }

  return (
    <DropdownMenu.Portal>
      <DropdownMenu.Content
        className={className}
        align={align}
        sideOffset={sideOffset}
      >
        {children}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  )
}

interface ItemProps {
  className?: string
  disabled?: boolean
  onSelect?: () => void
  children: ReactNode
  /** e.g. `data-testid` — forwarded to the underlying element as-is. */
  [dataAttribute: `data-${string}`]: string | undefined
}

/** A selectable row. Only needed for items that should close the menu on
 * click without the call site managing that itself — plain buttons (e.g.
 * ones that open a nested submenu or dialog) can be rendered directly. */
function Item({
  className,
  disabled = false,
  onSelect,
  children,
  ...rest
}: ItemProps) {
  const { isMobile, setOpen } = useResponsiveMenuContext('Item')

  if (isMobile) {
    return (
      <button
        type="button"
        role="menuitem"
        className={className}
        disabled={disabled}
        onClick={() => {
          onSelect?.()
          setOpen(false)
        }}
        {...rest}
      >
        {children}
      </button>
    )
  }

  return (
    <DropdownMenu.Item
      className={className}
      disabled={disabled}
      onSelect={onSelect}
      {...rest}
    >
      {children}
    </DropdownMenu.Item>
  )
}

interface LabelProps {
  className?: string
  children: ReactNode
}

function Label({ className, children }: LabelProps) {
  const { isMobile } = useResponsiveMenuContext('Label')
  return isMobile ? (
    <div className={className}>{children}</div>
  ) : (
    <DropdownMenu.Label className={className}>{children}</DropdownMenu.Label>
  )
}

export const ResponsiveMenu = { Root, Trigger, Content, Item, Label }
