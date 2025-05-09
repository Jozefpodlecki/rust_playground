import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";

interface Props {
  markdown: string;
}

const MarkdownViewer: React.FC<Props> = ({ markdown }) => {
  return (
    <div className="w-2/3 bg-gray-800 overflow-y-auto">
      <Markdown rehypePlugins={[rehypeHighlight]}>{markdown}</Markdown>
    </div>
  );
};

export default MarkdownViewer;
