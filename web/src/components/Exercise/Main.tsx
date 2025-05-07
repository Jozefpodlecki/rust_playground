import { useState, useEffect } from "react";
import TableOfContents from "./TableOfContents";
import MarkdownViewer from "./MarkdownViewer";
import InputPanel from "./InputPanel";
import { IconList } from "@tabler/icons-react";
import { useExercises } from "@/ExerciseProvider";
import { getMarkdown } from "@/api";

const Main: React.FC = () => {
	const [isDrawerOpen, setIsDrawerOpen] = useState(true);
	const [markdown, setMarkdown] = useState<string>("");
	const [tableOfContents, setTableOfContents] = useState<string[]>([]);
	const exercises = useExercises();

	useEffect(() => {
		
	}, []);

	async function onLoad() {
		const markdown = await getMarkdown(exercises.current!.id);
		setMarkdown(markdown);
	}

	return (
	<section className="flex h-full w-full bg-gray-900 text-white">
		<div className="flex flex-col w-full">
			{/* <div className="bg-gray-800 h-12 flex items-center justify-between px-6">
				<button onClick={() => setIsDrawerOpen(!isDrawerOpen)} className="flex items-center text-white">
				<IconList size={20} />
				<span className="ml-2">Table of Contents</span>
				</button>
				<div className="text-sm text-gray-300">v1.57.0 • v0.2.0</div>
			</div> */}

			<div className="flex h-full w-full p-6 gap-6">
				<TableOfContents open={isDrawerOpen} items={tableOfContents} />
				<MarkdownViewer markdown={markdown} />
				<InputPanel />
			</div>
		</div>
	</section>
	);
};

export default Main;
