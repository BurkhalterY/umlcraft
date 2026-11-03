// Marks <html data-os="macOS|Windows|Linux|FreeBSD"> before the page paints, so CSS can show the
// visitor's download first without a client re-render (the static HTML stays the same for all).
const SCRIPT = `(function(){var u=navigator.userAgent,p=(navigator.userAgentData&&navigator.userAgentData.platform)||navigator.platform||"",s=(p+" "+u).toLowerCase(),o="";
if(/freebsd/.test(s))o="FreeBSD";else if(/mac/.test(s)&&!/iphone|ipad/.test(s))o="macOS";else if(/win/.test(s))o="Windows";else if(/linux|x11/.test(s)&&!/android/.test(s))o="Linux";
if(o)document.documentElement.setAttribute("data-os",o);})();`;

export function OsScript() {
  return <script dangerouslySetInnerHTML={{ __html: SCRIPT }} />;
}

/** Classes: shown only when the visitor's system is `os` (rules in globals.css). */
export const onlyOn = (os: string) => `os-only os-${os}`;
/** Classes: shown only when the system is unknown. */
export const unknownOs = "os-unknown";
