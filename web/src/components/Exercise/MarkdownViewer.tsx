import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import remarkFrontmatter from "remark-frontmatter";
import remarkGfm from "remark-gfm";
import "github-markdown-css";
import { Box } from "@chakra-ui/react";

interface Props {
  markdown: string;
}

const MarkdownViewer: React.FC<Props> = ({ markdown }) => {
  return (
	<Box p="4" width={"2/3"} className="overflow-y-auto markdown-body">
		<Markdown
			remarkPlugins={[remarkFrontmatter]}
	  		rehypePlugins={[remarkGfm, rehypeHighlight]}>{markdown}</Markdown>
    </Box>
  );
};

export default MarkdownViewer;
