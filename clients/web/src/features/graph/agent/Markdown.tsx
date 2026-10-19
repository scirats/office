import ReactMarkdown from "react-markdown"
import remarkGfm from "remark-gfm"

export function Markdown({ children }: { children: string }) {
  return (
    <div className="prose prose-sm prose-invert max-w-none font-sans break-words prose-headings:mt-3 prose-headings:mb-1.5 prose-p:my-1.5 prose-pre:my-2 prose-pre:bg-background/60 prose-pre:text-xs prose-code:before:content-none prose-code:after:content-none prose-ul:my-1.5 prose-ol:my-1.5 prose-li:my-0.5">
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{children}</ReactMarkdown>
    </div>
  )
}
