// Text logo of the handy-diktier-app fork ("TypeTalk"), in the 3-t-s turquoise.
// The product name is not translated.
const PRODUCT_START = "Type";
const PRODUCT_END = "Talk";

const TypeTalkTextLogo = ({ className }: { className?: string }) => (
  <div
    className={className}
    style={{ fontWeight: 800, fontSize: "26px", letterSpacing: "-0.01em" }}
  >
    {PRODUCT_START}
    <span style={{ color: "#35d6dd" }}>{PRODUCT_END}</span>
  </div>
);

export default TypeTalkTextLogo;
