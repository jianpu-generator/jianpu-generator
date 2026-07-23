/** Shows the shared spinner alongside a button's label while `pending` is true. */
export function SpinnerLabel({
  pending,
  label,
}: {
  pending: boolean
  label: string
}) {
  return (
    <>
      {pending ? (
        <span
          className="file-tab-bar-spinner file-tab-bar-spinner--inline"
          aria-hidden="true"
        />
      ) : null}
      {label}
    </>
  )
}
