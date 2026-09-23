/** A tie arc joining two same-pitch note heads, drawn in lucide-react's
 * style (24×24 viewBox, `currentColor` 2px round strokes) since lucide has
 * no tie icon — used by the editor toolbar's "Tie/Untie" button. Flatter
 * and lower than `SlurIcon`'s arc, with the heads sitting level beneath its
 * ends, to read as a tie rather than a phrase slur. */
export function TieIcon({ size = 24 }: { size?: number }) {
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
      <path d="M5 16 Q12 10 19 16" />
      <circle cx="5" cy="19" r="1.5" fill="currentColor" />
      <circle cx="19" cy="19" r="1.5" fill="currentColor" />
    </svg>
  )
}
