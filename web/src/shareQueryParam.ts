// The query param a shared-score link mirrors its `#synced=` hash payload
// into. A crawler that generates a link preview (WhatsApp, Slack, ...) never
// runs JS and never sends the URL fragment to the server, so the Pages
// Function `functions/index.ts` -- which needs the share id to fetch the
// real title before the crawler ever sees the HTML -- reads this instead.
// The hash remains the only thing the client itself reads; this param exists
// purely for server-side consumption.
//
// Kept in its own dependency-free module (rather than in `syncedShareUrl.ts`,
// which touches `window`/`import.meta.env`) so both the SPA and that edge
// function import this one definition without pulling SPA code into the
// function's bundle.
export const SHARE_QUERY_PARAM = 's'
