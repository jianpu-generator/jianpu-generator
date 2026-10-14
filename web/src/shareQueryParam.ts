// The query param a synced-share link carries its payload
// (`<shareId>[--<filename>]`) in. The client reads it, and so does the Pages
// Function `functions/index.ts`: a crawler that generates a link preview
// (WhatsApp, Slack, ...) never runs JS and never sends the URL fragment to
// the server, so the function needs the share id in the query to fetch the
// real title before the crawler ever sees the HTML.
//
// Kept in its own dependency-free module (rather than in `syncedShareUrl.ts`,
// which touches `window`/`import.meta.env`) so both the SPA and that edge
// function import this one definition without pulling SPA code into the
// function's bundle.
export const SHARE_QUERY_PARAM = 's'
