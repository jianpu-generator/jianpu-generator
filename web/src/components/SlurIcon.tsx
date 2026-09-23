/** A slur arc over two note heads, drawn in lucide-react's style (24×24
 * viewBox, `currentColor` 2px round strokes) since lucide has no slur icon —
 * used by the editor toolbar's "Slur/Unslur" button. */
export function SlurIcon({ size = 24 }: { size?: number }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M4 13 Q12 3 20 13" />
      <circle cx="6" cy="19" r="1.5" fill="currentColor" />
      <circle cx="18" cy="19" r="1.5" fill="currentColor" />
    </svg>
  )
}
